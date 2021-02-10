import { Theme, createMuiTheme, responsiveFontSizes } from "@material-ui/core";

function theme(): Theme {
  const theme = responsiveFontSizes(
    createMuiTheme({
      palette: {
        type: "dark",
        primary: {
          light: "#99d066",
          main: "#689f38",
          dark: "#387002",
        },
        secondary: {
          light: "#5778d5",
          main: "#144da3",
          dark: "#002673",
        },
        background: {
          default: "#4a3424",
          paper: "#775e4c",
        },
      },

      typography: {
        // Makes math for `rem` font sizes easy
        // https://www.sitepoint.com/understanding-and-using-rem-units-in-css/
        htmlFontSize: 10,
        fontFamily: "sans-serif",

        h1: {
          fontSize: "3.2rem",
          margin: "1rem 0",
        },
        h2: {
          fontSize: "2.8rem",
          margin: "1rem 0",
        },
        h3: {
          fontSize: "2.4rem",
          margin: "0.875rem 0",
        },
        h4: {
          fontSize: "2.0rem",
        },
        h5: {
          fontSize: "1.6rem",
        },
        h6: {
          fontSize: "1.2rem",
        },
        body1: {
          margin: "1rem 0",
        },
        button: {
          textTransform: "none",
        },
      },
    })
  );

  theme.overrides = {
    MuiTooltip: {
      tooltip: {
        backgroundColor: theme.palette.background.paper,
        "& a": {
          color: theme.palette.primary.light,
        },
        "& ol": {
          paddingLeft: theme.spacing(4),
        },
      },
      arrow: {
        color: theme.palette.background.paper,
      },
    },
  };

  return theme;
}

export default theme;
