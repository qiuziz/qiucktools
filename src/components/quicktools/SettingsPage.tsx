import { useTranslation } from "react-i18next";
import { useState } from "react";
import { Moon, Sun, Monitor, Languages } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

const languages = [
  { value: "en", label: "English" },
  { value: "zh", label: "中文" },
  { value: "ja", label: "日本語" },
];

const themes = [
  { value: "light", icon: Sun, label: "Light" },
  { value: "dark", icon: Moon, label: "Dark" },
  { value: "system", icon: Monitor, label: "System" },
];

export function SettingsPage() {
  const { t, i18n } = useTranslation();
  const [theme, setTheme] = useState(
    localStorage.getItem("quicktools-theme") || "system"
  );
  const [language, setLanguage] = useState(i18n.language || "en");

  const handleThemeChange = (value: string) => {
    setTheme(value);
    document.documentElement.classList.remove("light", "dark");
    if (value !== "system") {
      document.documentElement.classList.add(value);
    }
    localStorage.setItem("quicktools-theme", value);
  };

  const handleLanguageChange = (value: string) => {
    setLanguage(value);
    i18n.changeLanguage(value);
    localStorage.setItem("i18nextLng", value);
  };

  return (
    <div className="max-w-2xl mx-auto space-y-6">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Languages className="h-5 w-5" />
            {t("settings.language", "Language")}
          </CardTitle>
          <CardDescription>
            {t("settings.languageDesc", "Select your preferred language")}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex gap-4">
            {languages.map((lang) => (
              <Button
                key={lang.value}
                variant={language === lang.value ? "default" : "outline"}
                onClick={() => handleLanguageChange(lang.value)}
              >
                {lang.label}
              </Button>
            ))}
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{t("settings.theme", "Theme")}</CardTitle>
          <CardDescription>
            {t("settings.themeDesc", "Choose your preferred appearance")}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex gap-4">
            {themes.map((th) => {
              const Icon = th.icon;
              return (
                <Button
                  key={th.value}
                  variant={theme === th.value ? "default" : "outline"}
                  onClick={() => handleThemeChange(th.value)}
                >
                  <Icon className="h-4 w-4 mr-2" />
                  {th.label}
                </Button>
              );
            })}
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>{t("settings.about", "About")}</CardTitle>
        </CardHeader>
        <CardContent className="space-y-2">
          <p className="text-sm text-muted-foreground">
            <strong>QuickTools</strong> v0.1.0
          </p>
          <p className="text-sm text-muted-foreground">
            {t("app.description", "A tool execution utility for macOS")}
          </p>
          <p className="text-xs text-muted-foreground mt-4">
            {t("settings.toolsConfig", "Tools config")}
            : ~/work/quicktools/tools.json
          </p>
        </CardContent>
      </Card>
    </div>
  );
}