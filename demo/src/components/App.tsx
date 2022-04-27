import { CssBaseline, ThemeProvider } from "@material-ui/core";
import React from "react";
import { HashRouter } from "react-router-dom";
import Demo from "./demo/Demo";
import PageLayout from "./PageLayout";
import theme from "../theme";

const App: React.FC = () => {
  return (
    <ThemeProvider theme={theme()}>
      <CssBaseline />
      {/* Deployment is a lot more difficult with BrowserRouter because you have
      to redirect 404s to index.html. HashRouter lets us stick with GitHub Pages */}
      <HashRouter>
        <PageLayout>
          <Demo />
        </PageLayout>
      </HashRouter>
    </ThemeProvider>
  );
};

export default App;
