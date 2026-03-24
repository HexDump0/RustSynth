import { useCallback, useEffect } from "react";
import { useShepherd } from "react-shepherd";
import { offset } from "@floating-ui/dom";

const ONBOARDING_SEEN_KEY = "onboarding";

export function useRustSynthOnboarding() {
    const Shepherd = useShepherd();

    const markOnboardingSeen = useCallback(() => {
        try {
            window.localStorage.setItem(ONBOARDING_SEEN_KEY, "1");
        } catch {}
    }, []);

    const startOnboarding = useCallback(() => {
        const floatingUIOptions = {
            middleware: [offset({ mainAxis: 10, crossAxis: 0 })],
        };

        const tour = new Shepherd.Tour({
            useModalOverlay: true,
            defaultStepOptions: {
                cancelIcon: { enabled: false },
                classes: "custom-theme",
                scrollTo: { behavior: "smooth", block: "center" },
            },
        });

        const nextButton = { text: "Next", action: tour.next };
        const backButton = { text: "Back", action: tour.back };

        tour.addStep({
            id: "welcome",
            title: "Welcome to RustSynth",
            text: "RustSynth is a rewrite of Structure Synth that allows you to create beautiful and complex 3D models with code.",
            buttons: [{ text: "Skip", action: tour.cancel }, nextButton],
        });

        tour.addStep({
            id: "examples",
            title: "Examples",
            text: "Try out some example scripts to get a feel for what RustSynth can do.",
            attachTo: { element: "#tour-examples", on: "bottom" },
            floatingUIOptions,
            buttons: [backButton, nextButton],
        });

        tour.addStep({
            id: "editor",
            title: "Script editor",
            text: "This is where you will write your EisenScript code. Make sure to check the docs to learn about EisenScript \n TIP: Use CTRL+ENTER to run your code",
            attachTo: { element: "#tour-editor", on: "right" },
            floatingUIOptions,
            buttons: [backButton, nextButton],
        });

        tour.addStep({
            id: "run",
            title: "Run and export",
            text: "Build your model with Run, or export it as an OBJ file.",
            attachTo: { element: "#tour-run", on: "bottom" },
            floatingUIOptions,
            buttons: [backButton, nextButton],
        });

        tour.addStep({
            id: "viewport",
            title: "viewport",
            text: "Rotate with left mouse, Move with right mouse and Zoom with scroll wheel. Use camera buttons at the bottom to insert or reset the camera position.",
            attachTo: { element: "#tour-viewport", on: "left" },
            floatingUIOptions,
            buttons: [backButton, nextButton],
        });

        tour.addStep({
            id: "status",
            title: "Status",
            text: "Toggle the console and monitor the object count using the status bar.",
            attachTo: { element: "#tour-status", on: "top" },
            floatingUIOptions,
            buttons: [backButton, nextButton],
        });

        tour.addStep({
            id: "docs",
            title: "Docs",
            text: "Has pretty much everything you would need to know.",
            attachTo: { element: "#tour-docs", on: "top" },
            floatingUIOptions,
            buttons: [
                backButton,
                {
                    text: "Done",
                    action: tour.complete,
                },
            ],
        });

        tour.on("complete", markOnboardingSeen);
        tour.on("cancel", markOnboardingSeen);
        tour.start();
    }, [Shepherd, markOnboardingSeen]);

    useEffect(() => {
        let hasSeenTour = false;
        try {
            hasSeenTour =
                window.localStorage.getItem(ONBOARDING_SEEN_KEY) === "1";
        } catch {
            hasSeenTour = false;
        }

        if (hasSeenTour) {
            return;
        }

        const timeoutId = window.setTimeout(() => {
            startOnboarding();
        }, 500);

        return () => {
            window.clearTimeout(timeoutId);
        };
    }, [startOnboarding]);

    return {
        startOnboarding,
    };
}
