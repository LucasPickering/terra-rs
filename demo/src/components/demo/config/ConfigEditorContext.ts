import React from "react";

// TODO figure out a way to get rid of this context. Ideally we could have
// ConfigSection size itself based on the parent, then we don't ever need to
// specify fullscreen vs inline. Alternatively, each config section could look
// at the DemoContext to figure out if they need to be fullscreen-sized or not.

export interface ConfigEditorContextType {
  fullscreen: boolean;
}

const ConfigEditorContext = React.createContext<ConfigEditorContextType>(
  {} as ConfigEditorContextType // Safe because this value never gets used
);

export default ConfigEditorContext;
