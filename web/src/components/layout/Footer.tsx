export function Footer() {
  const currentYear = new Date().getFullYear();

  return (
    <footer className="footer-redesign">
      <div className="footer-top">
        <div className="footer-section">
          <h3 className="footer-section-title">Resources</h3>
          <a href="/docs/whitepaper.pdf" target="_blank" rel="noopener noreferrer">
            Whitepaper <span className="sr-only">(opens in new tab)</span>
          </a>
          <a href="https://github.com/ecto/muni" target="_blank" rel="noopener noreferrer">
            GitHub <span className="sr-only">(opens in new tab)</span>
          </a>
          <a href="https://github.com/ecto/muni/tree/main/bvr/docs/hardware" target="_blank" rel="noopener noreferrer">
            Build Guide <span className="sr-only">(opens in new tab)</span>
          </a>
        </div>

        <div className="footer-section">
          <h3 className="footer-section-title">Contact</h3>
          <a href="mailto:info@muni.works">info@muni.works</a>
          <a href="mailto:info@muni.works?subject=Pilot%20program">Pilot Program</a>
        </div>

        <div className="footer-section">
          <h3 className="footer-section-title">Social</h3>
          <a href="https://x.com/municipalrobots" target="_blank" rel="noopener noreferrer">
            X <span className="sr-only">(opens in new tab)</span>
          </a>
          <a href="https://www.linkedin.com/company/municipal-robotics" target="_blank" rel="noopener noreferrer">
            LinkedIn <span className="sr-only">(opens in new tab)</span>
          </a>
        </div>
      </div>

      <div className="footer-bottom">
        <p>© {currentYear} Municipal Robotics · Cleveland, Ohio</p>
        <p className="footer-tagline">Autonomous utility vehicles for public works</p>
        <p className="footer-investors-link">
          <a href="/investors">Investor Information</a>
        </p>
      </div>
    </footer>
  );
}
