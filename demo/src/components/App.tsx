import { CssBaseline, ThemeProvider } from "@material-ui/core";
import React from "react";
import { BrowserRouter, Switch, Route, Redirect } from "react-router-dom";
import Demo from "./demo/Demo";
import PageLayout from "./PageLayout";
import NotFound from "./NotFound";
import theme from "../theme";

const App: React.FC = () => {
  return (
    <ThemeProvider theme={theme()}>
      <CssBaseline />
      <BrowserRouter>
        <PageLayout>
          <Switch>
            <Route path="/" exact>
              <Redirect from="/" to="/demo/new" exact />
            </Route>

            <Route path="/demo">
              <Demo />
            </Route>

            <Route path="*">
              <NotFound />
            </Route>
          </Switch>
        </PageLayout>
      </BrowserRouter>
    </ThemeProvider>
  );
};

export default App;
