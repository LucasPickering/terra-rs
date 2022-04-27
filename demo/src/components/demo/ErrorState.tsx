import { Typography } from "@material-ui/core";
import Link from "components/Link";
import React from "react";

const ErrorState: React.FC = () => (
  <div>
    <Typography variant="h3">
      An error occurred during world generation: (
    </Typography>
    <Typography component="div">
      Please{" "}
      <Link to="https://github.com/LucasPickering/terra-rs/issues/new">
        file an issue
      </Link>{" "}
      for this and <strong>include the following:</strong>
      <ul>
        <li>
          e world generation config JSON (available in the config editor panel
          on this page)
        </li>
        <li>The error from the browser developer console</li>
      </ul>
    </Typography>
  </div>
);

export default ErrorState;
