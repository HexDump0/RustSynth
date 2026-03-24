import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { ShepherdJourneyProvider } from "react-shepherd";
import App from "./App";
import "shepherd.js/dist/css/shepherd.css";
import "./styles.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <ShepherdJourneyProvider>
      <App />
    </ShepherdJourneyProvider>
  </StrictMode>,
);
