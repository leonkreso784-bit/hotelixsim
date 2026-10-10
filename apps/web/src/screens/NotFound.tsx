import { t } from "@hotelixsim/i18n";
import { Link } from "../router";
import { Card } from "../ui/primitives";

export function NotFound() {
  return (
    <Card>
      <h1 className="text-xl font-bold">{t("notFound.title")}</h1>
      <Link to="/" className="mt-3 inline-block font-semibold text-accent">
        {t("notFound.back")}
      </Link>
    </Card>
  );
}
