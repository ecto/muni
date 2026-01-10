import Link from "next/link";

const navItems = [
  { href: "/about", label: "ABOUT" },
  { href: "/products", label: "PRODUCTS" },
  { href: "/investors", label: "INVESTORS" },
  { href: "/docs", label: "DOCS" },
];

export function NavBar() {
  return (
    <nav className="nav-bar">
      {navItems.map((item) => (
        <Link key={item.href} href={item.href}>
          {item.label}
        </Link>
      ))}
    </nav>
  );
}
