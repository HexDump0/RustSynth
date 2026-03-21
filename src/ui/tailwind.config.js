/** @type {import('tailwindcss').Config} */
module.exports = {
    content: [
        "./**/*.html",
        "./src/**/*.js",
        "./src/**/*.ts",
        "./src/**/*.jsx",
        "./src/**/*.tsx",
    ],
    theme: {
        extend: {
            colors: {
                dark: "#07070a",
                "dark-1": "#0e0e14",
                "dark-2": "#1a1a22",
                "dark-3": "#2a2a34",
                "dark-4": "#3a3a44",
                "dark-5": "#4a4a54",

                primary: "#89b4fa",
                secondary: "#a6e3a1",

                text: "#e4e7ef",
                muted: "#9399b2",

                // Legacy compatibility
                dark: {
                    50: "#cdd6f4",
                    100: "#cdd6f4",
                    200: "#a6adc8",
                    300: "#9399b2",
                    400: "#6c7086",
                    500: "#5c6075",
                    600: "#45475a",
                    700: "#313244",
                    800: "#1e1e2e",
                    900: "#07070a",
                },
            },
            fontFamily: {
                sans: ["Roboto", "system-ui", "sans-serif"],
                mono: ["JetBrains Mono", "Fira Code", "monospace"],
            },
            animation: {
                "fade-in": "fadeIn 0.3s ease-in-out",
                "slide-up": "slideUp 0.3s ease-out",
                "slide-down": "slideDown 0.3s ease-out",
            },
            keyframes: {
                fadeIn: {
                    "0%": { opacity: "0" },
                    "100%": { opacity: "1" },
                },
                slideUp: {
                    "0%": { transform: "translateY(10px)", opacity: "0" },
                    "100%": { transform: "translateY(0)", opacity: "1" },
                },
                slideDown: {
                    "0%": { transform: "translateY(-10px)", opacity: "0" },
                    "100%": { transform: "translateY(0)", opacity: "1" },
                },
            },
        },
    },
    plugins: [],
};