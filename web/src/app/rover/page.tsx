import type { Metadata } from "next";
import {
  ArrowRight,
  GithubLogo,
  FileText,
  Wrench,
  Shield,
  Cpu,
  BatteryFull,
  Broadcast,
  MapPin,
  Eye,
  Gear,
} from "@phosphor-icons/react/dist/ssr";
import { Footer, FloatingHeader } from "@/components/layout";

export const metadata: Metadata = {
  title: "BVR1 Rover | Muni",
  description:
    "BVR1 autonomous sidewalk rover. 50 miles per night. Zero labor cost.",
  alternates: {
    canonical: "https://muni.works/rover",
  },
};

const productSchema = {
  "@context": "https://schema.org",
  "@type": "Product",
  name: "BVR1 Autonomous Sidewalk Rover",
  description:
    "Fully autonomous sidewalk clearing robot. 4-wheel skid-steer platform with Jetson Orin NX, Livox LiDAR, and modular tool attachments.",
  image: "https://muni.works/images/bvr1.png",
  brand: {
    "@type": "Brand",
    name: "Municipal Robotics",
  },
};

export default function RoverPage() {
  return (
    <>
      <FloatingHeader />
      <main className="products">
        {/* Hero */}
        <section className="products-hero">
          <div className="products-container">
            <div className="products-hero-content">
              <div className="products-hero-text">
                <span className="products-eyebrow">BVR1 Production Rover</span>
                <h1 className="products-title">
                  Autonomous sidewalk clearing.
                </h1>
                <p className="products-subtitle">
                  50 miles per night. Zero labor cost. Operates in active snowfall.
                </p>
                <div className="products-hero-cta">
                  <a href="https://muni.cal.com/cam/30min" className="products-btn products-btn-primary">
                    Schedule a Call
                    <ArrowRight size={16} weight="bold" />
                  </a>
                  <a href="/docs/whitepaper.pdf" className="products-btn products-btn-secondary" target="_blank" rel="noopener noreferrer">
                    Read Whitepaper
                  </a>
                </div>
              </div>
              <div className="products-hero-image">
                <img src="/images/bvr1.png" alt="BVR1 Production Rover" />
              </div>
            </div>
          </div>
        </section>

        {/* Specs */}
        <section className="products-section">
          <div className="products-container">
            <h2 className="products-section-title">Specifications</h2>

            <div className="products-specs-grid">
              <div className="products-spec">
                <div className="products-spec-icon"><Cpu size={20} weight="bold" /></div>
                <div className="products-spec-content">
                  <span className="products-spec-label">Compute</span>
                  <span className="products-spec-value">Jetson Orin NX</span>
                  <span className="products-spec-detail">30 TOPS AI performance</span>
                </div>
              </div>

              <div className="products-spec">
                <div className="products-spec-icon"><Eye size={20} weight="bold" /></div>
                <div className="products-spec-content">
                  <span className="products-spec-label">LiDAR</span>
                  <span className="products-spec-value">Livox Mid-360</span>
                  <span className="products-spec-detail">360° × 59° FOV, 200k pts/sec</span>
                </div>
              </div>

              <div className="products-spec">
                <div className="products-spec-icon"><BatteryFull size={20} weight="bold" /></div>
                <div className="products-spec-content">
                  <span className="products-spec-label">Battery</span>
                  <span className="products-spec-value">48V 40Ah LiFePO4</span>
                  <span className="products-spec-detail">4-8 hour runtime</span>
                </div>
              </div>

              <div className="products-spec">
                <div className="products-spec-icon"><Broadcast size={20} weight="bold" /></div>
                <div className="products-spec-content">
                  <span className="products-spec-label">Connectivity</span>
                  <span className="products-spec-value">LTE + WiFi</span>
                  <span className="products-spec-detail">100-250ms typical latency</span>
                </div>
              </div>

              <div className="products-spec">
                <div className="products-spec-icon"><MapPin size={20} weight="bold" /></div>
                <div className="products-spec-content">
                  <span className="products-spec-label">Positioning</span>
                  <span className="products-spec-value">RTK GPS + IMU</span>
                  <span className="products-spec-detail">Centimeter accuracy</span>
                </div>
              </div>

              <div className="products-spec">
                <div className="products-spec-icon"><Gear size={20} weight="bold" /></div>
                <div className="products-spec-content">
                  <span className="products-spec-label">Tools</span>
                  <span className="products-spec-value">Hot-swap CAN bus</span>
                  <span className="products-spec-detail">Auger, spreader, plow</span>
                </div>
              </div>
            </div>

            <div className="products-specs-table">
              <div className="products-specs-row">
                <span>Platform</span>
                <span>600mm × 600mm × 400mm</span>
              </div>
              <div className="products-specs-row">
                <span>Weight</span>
                <span>~60 kg (132 lbs)</span>
              </div>
              <div className="products-specs-row">
                <span>Speed</span>
                <span>0-1 m/s (0-2.2 mph)</span>
              </div>
              <div className="products-specs-row">
                <span>Camera</span>
                <span>Insta360 X4 (360° video)</span>
              </div>
              <div className="products-specs-row">
                <span>Warranty</span>
                <span>1 year parts and labor</span>
              </div>
            </div>
          </div>
        </section>

        {/* Safety */}
        <section className="products-section">
          <div className="products-container">
            <h2 className="products-section-title">Safety Systems</h2>

            <div className="products-safety-grid">
              <div className="products-safety-item">
                <Shield size={20} weight="bold" />
                <div>
                  <strong>1.5m Safety Radius</strong>
                  <p>LiDAR-based obstacle detection. Immediate e-stop if breached.</p>
                </div>
              </div>
              <div className="products-safety-item">
                <Shield size={20} weight="bold" />
                <div>
                  <strong>Watchdog Timer</strong>
                  <p>Auto e-stop on connection loss (250ms timeout).</p>
                </div>
              </div>
              <div className="products-safety-item">
                <Shield size={20} weight="bold" />
                <div>
                  <strong>Rate Limiting</strong>
                  <p>Prevents dangerous acceleration commands.</p>
                </div>
              </div>
              <div className="products-safety-item">
                <Shield size={20} weight="bold" />
                <div>
                  <strong>No ML in Safety Path</strong>
                  <p>Pure geometry. Deterministic behavior.</p>
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* Resources */}
        <section className="products-section">
          <div className="products-container">
            <h2 className="products-section-title">Resources</h2>

            <div className="products-resources">
              <a href="/docs/whitepaper.pdf" className="products-resource" target="_blank" rel="noopener noreferrer">
                <FileText size={24} weight="bold" />
                <div>
                  <strong>Whitepaper</strong>
                  <span>Technical overview and economics</span>
                </div>
                <ArrowRight size={16} weight="bold" />
              </a>

              <a href="https://github.com/ecto/muni" className="products-resource" target="_blank" rel="noopener noreferrer">
                <GithubLogo size={24} weight="bold" />
                <div>
                  <strong>Source Code</strong>
                  <span>Firmware, CAD, schematics</span>
                </div>
                <ArrowRight size={16} weight="bold" />
              </a>

              <a href="/docs/bvr0-manual.pdf" className="products-resource" target="_blank" rel="noopener noreferrer">
                <Wrench size={24} weight="bold" />
                <div>
                  <strong>BVR0 Manual</strong>
                  <span>Assembly and operation guide</span>
                </div>
                <ArrowRight size={16} weight="bold" />
              </a>
            </div>
          </div>
        </section>

        {/* CTA */}
        <section className="products-section products-cta">
          <div className="products-container">
            <div className="products-cta-content">
              <span className="products-eyebrow">Shipping Summer 2026</span>
              <h2 className="products-cta-title">Ready to get started?</h2>
              <p className="products-cta-desc">
                Schedule a call to discuss your deployment needs.
              </p>
              <a href="https://muni.cal.com/cam/30min" className="products-btn products-btn-primary products-btn-large">
                Schedule a Call
                <ArrowRight size={18} weight="bold" />
              </a>
            </div>
          </div>
        </section>

        {/* Footer */}
        <footer className="products-footer">
          <div className="products-container">
            <Footer />
          </div>
        </footer>
      </main>

      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: JSON.stringify(productSchema) }}
      />
    </>
  );
}
