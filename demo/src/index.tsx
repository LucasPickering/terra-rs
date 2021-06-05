import React, { Suspense } from "react";
import ReactDOM from "react-dom";
import "./index.css";
const App = React.lazy(() => import("./components/App"));

ReactDOM.render(
  <React.StrictMode>
    <Suspense fallback={null}>
      <App />
    </Suspense>
  </React.StrictMode>,
  document.getElementById("root")
);
