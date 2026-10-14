import "./styles.css";
import { branding } from "./branding.generated";
import { t } from "./i18n";

document.title = branding.name;
document.documentElement.style.setProperty("--accent", branding.accentColor);

document.querySelector<HTMLElement>("#app")!.innerHTML = `
  <h1>${t("welcome.title", { name: branding.name })}</h1>
  <p>${t("welcome.subtitle")}</p>
`;
