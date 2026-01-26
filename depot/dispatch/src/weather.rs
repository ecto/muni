//! NOAA Weather API integration.
//!
//! Fetches hourly forecasts from the National Weather Service API and caches
//! them in memory + PostgreSQL. Provides a REST endpoint for the console
//! dashboard and triggers weather-based alerts.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::SharedState;

// =============================================================================
// Public Types (API Response)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherResponse {
    pub current: Option<CurrentConditions>,
    pub forecast: Vec<ForecastPeriod>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentConditions {
    pub temperature_f: f64,
    pub wind_speed_mph: f64,
    pub wind_direction: String,
    pub short_forecast: String,
    pub humidity_pct: Option<f64>,
    pub is_snowing: bool,
    pub icon_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForecastPeriod {
    pub start_time: String,
    pub end_time: String,
    pub temperature_f: f64,
    pub wind_speed: String,
    pub wind_direction: String,
    pub short_forecast: String,
    pub precipitation_probability: Option<f64>,
    pub is_snow: bool,
}

// =============================================================================
// NOAA API Types
// =============================================================================

#[derive(Debug, Deserialize)]
struct PointsResponse {
    properties: PointsProperties,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PointsProperties {
    forecast_hourly: String,
    #[allow(dead_code)]
    forecast: String,
}

#[derive(Debug, Deserialize)]
struct ForecastResponse {
    properties: ForecastProperties,
}

#[derive(Debug, Deserialize)]
struct ForecastProperties {
    periods: Vec<NoaaPeriod>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NoaaPeriod {
    start_time: String,
    end_time: String,
    temperature: f64,
    #[serde(default)]
    temperature_unit: String,
    wind_speed: String,
    wind_direction: String,
    short_forecast: String,
    #[serde(default)]
    probability_of_precipitation: Option<PrecipProb>,
    #[serde(default)]
    relative_humidity: Option<HumidityValue>,
    #[serde(default)]
    icon: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PrecipProb {
    value: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct HumidityValue {
    value: Option<f64>,
}

// =============================================================================
// Weather Cache
// =============================================================================

pub struct WeatherCache {
    pub data: RwLock<Option<WeatherResponse>>,
    /// NOAA hourly forecast URL (resolved from lat/lng on first fetch)
    forecast_url: RwLock<Option<String>>,
}

impl WeatherCache {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            data: RwLock::new(None),
            forecast_url: RwLock::new(None),
        })
    }
}

// =============================================================================
// REST Endpoint
// =============================================================================

/// GET /api/weather — current conditions + 24h forecast
pub async fn get_weather(State(state): State<SharedState>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let cache = state.weather_cache.data.read().await;
    match cache.as_ref() {
        Some(data) => Ok(Json(data.clone())),
        None => {
            // Try loading from DB
            let row: Option<(serde_json::Value,)> = sqlx::query_as(
                "SELECT data FROM weather_cache ORDER BY fetched_at DESC LIMIT 1",
            )
            .fetch_optional(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            match row {
                Some((data,)) => {
                    let weather: WeatherResponse = serde_json::from_value(data)
                        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
                    Ok(Json(weather))
                }
                None => Ok(Json(WeatherResponse {
                    current: None,
                    forecast: vec![],
                    updated_at: None,
                })),
            }
        }
    }
}

// =============================================================================
// Background Fetch Loop
// =============================================================================

/// Fetch weather from NOAA every 30 minutes and cache results.
pub async fn weather_fetcher(state: SharedState) {
    use std::time::Duration;
    use tokio::time::interval;

    let lat = std::env::var("DEPOT_LATITUDE")
        .ok()
        .and_then(|v| v.parse::<f64>().ok());
    let lng = std::env::var("DEPOT_LONGITUDE")
        .ok()
        .and_then(|v| v.parse::<f64>().ok());

    let (lat, lng) = match (lat, lng) {
        (Some(lat), Some(lng)) => (lat, lng),
        _ => {
            info!("DEPOT_LATITUDE/DEPOT_LONGITUDE not set, weather fetching disabled");
            return;
        }
    };

    info!(lat, lng, "Starting weather fetch loop");

    // Initial fetch after 5 seconds, then every 30 minutes
    tokio::time::sleep(Duration::from_secs(5)).await;

    let mut ticker = interval(Duration::from_secs(1800));

    loop {
        match fetch_weather(&state, lat, lng).await {
            Ok(weather) => {
                // Update in-memory cache
                *state.weather_cache.data.write().await = Some(weather.clone());

                // Persist to DB
                let json = serde_json::to_value(&weather).unwrap_or_default();
                let _ = sqlx::query(
                    "INSERT INTO weather_cache (data, fetched_at) VALUES ($1, now())",
                )
                .bind(&json)
                .execute(&state.db)
                .await;

                // Trim old cache entries (keep last 48)
                let _ = sqlx::query(
                    r#"
                    DELETE FROM weather_cache
                    WHERE id NOT IN (
                        SELECT id FROM weather_cache ORDER BY fetched_at DESC LIMIT 48
                    )
                    "#,
                )
                .execute(&state.db)
                .await;

                // Weather-based alerts
                check_weather_alerts(&state, &weather).await;

                info!("Weather updated successfully");
            }
            Err(e) => {
                warn!(error = %e, "Failed to fetch weather");
            }
        }

        ticker.tick().await;
    }
}

async fn fetch_weather(
    state: &SharedState,
    lat: f64,
    lng: f64,
) -> Result<WeatherResponse, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .user_agent("muni-dispatch/0.1 (github.com/muni-robotics/muni)")
        .timeout(std::time::Duration::from_secs(15))
        .build()?;

    // Resolve forecast URL if not cached
    let forecast_url = {
        let cached = state.weather_cache.forecast_url.read().await;
        cached.clone()
    };

    let forecast_url = match forecast_url {
        Some(url) => url,
        None => {
            let points_url = format!(
                "https://api.weather.gov/points/{:.4},{:.4}",
                lat, lng
            );
            let points: PointsResponse = client.get(&points_url).send().await?.json().await?;
            let url = points.properties.forecast_hourly;
            *state.weather_cache.forecast_url.write().await = Some(url.clone());
            url
        }
    };

    // Fetch hourly forecast
    let forecast: ForecastResponse = client.get(&forecast_url).send().await?.json().await?;
    let periods = forecast.properties.periods;

    // First period is "current"
    let current = periods.first().map(|p| {
        let is_snowing = p.short_forecast.to_lowercase().contains("snow");
        CurrentConditions {
            temperature_f: p.temperature,
            wind_speed_mph: parse_wind_speed(&p.wind_speed),
            wind_direction: p.wind_direction.clone(),
            short_forecast: p.short_forecast.clone(),
            humidity_pct: p.relative_humidity.as_ref().and_then(|h| h.value),
            is_snowing,
            icon_url: p.icon.clone(),
        }
    });

    // Next 24 periods (24 hours)
    let forecast_periods: Vec<ForecastPeriod> = periods
        .iter()
        .take(24)
        .map(|p| {
            let is_snow = p.short_forecast.to_lowercase().contains("snow");
            ForecastPeriod {
                start_time: p.start_time.clone(),
                end_time: p.end_time.clone(),
                temperature_f: p.temperature,
                wind_speed: p.wind_speed.clone(),
                wind_direction: p.wind_direction.clone(),
                short_forecast: p.short_forecast.clone(),
                precipitation_probability: p
                    .probability_of_precipitation
                    .as_ref()
                    .and_then(|pp| pp.value),
                is_snow,
            }
        })
        .collect();

    Ok(WeatherResponse {
        current,
        forecast: forecast_periods,
        updated_at: Some(Utc::now()),
    })
}

fn parse_wind_speed(s: &str) -> f64 {
    // NOAA formats: "10 mph", "5 to 10 mph"
    s.split_whitespace()
        .find_map(|word| word.parse::<f64>().ok())
        .unwrap_or(0.0)
}

async fn check_weather_alerts(state: &SharedState, weather: &WeatherResponse) {
    use crate::alerts::{
        broadcast_alert, clear_alerts_matching, create_alert, has_active_alert, AlertBroadcast,
    };

    // Check for snow in forecast
    let snow_periods: Vec<&ForecastPeriod> = weather
        .forecast
        .iter()
        .filter(|p| p.is_snow)
        .collect();

    if !snow_periods.is_empty() {
        let hours = snow_periods.len();
        let message = format!("Snow expected in next {}h", hours);

        if let Ok(false) = has_active_alert(&state.db, "weather", "forecast", &message).await {
            if let Ok(alert) = create_alert(
                &state.db, "info", "weather", "forecast", &message, None,
            ).await {
                broadcast_alert(state, AlertBroadcast::AlertCreated { alert });
            }
        }
    } else {
        // Clear snow forecast alerts if no more snow
        if let Ok(cleared) =
            clear_alerts_matching(&state.db, "weather", "forecast", "Snow expected%").await
        {
            for id in cleared {
                broadcast_alert(state, AlertBroadcast::AlertCleared { id });
            }
        }
    }

    // Active snow + no active missions = warning
    if let Some(ref current) = weather.current {
        if current.is_snowing {
            let active_tasks: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM tasks WHERE status IN ('assigned', 'active')",
            )
            .fetch_one(&state.db)
            .await
            .unwrap_or(0);

            if active_tasks == 0 {
                let message = "Snow falling, no active missions";
                if let Ok(false) =
                    has_active_alert(&state.db, "weather", "active", message).await
                {
                    if let Ok(alert) = create_alert(
                        &state.db, "warning", "weather", "active", message, None,
                    ).await {
                        broadcast_alert(state, AlertBroadcast::AlertCreated { alert });
                    }
                }
            }
        } else {
            // Clear "snow falling" alerts when it stops
            if let Ok(cleared) =
                clear_alerts_matching(&state.db, "weather", "active", "Snow falling%").await
            {
                for id in cleared {
                    broadcast_alert(state, AlertBroadcast::AlertCleared { id });
                }
            }
        }
    }
}
