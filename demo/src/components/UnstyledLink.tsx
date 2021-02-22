import React from "react";
import { Link as RouterLink } from "react-router-dom";

type Props = React.ComponentProps<typeof RouterLink>;

const UnstyledLink = React.forwardRef(
  (
    { to, children, ...rest }: Props,
    ref: React.Ref<HTMLAnchorElement>
  ): React.ReactElement => {
    const destString = to.toString();
    const external = Boolean(destString.match(/^https?:/));
    const apiLink = Boolean(destString.match(/^\/api\//));

    if (external || apiLink) {
      return (
        <a
          ref={ref}
          href={destString}
          {...(external
            ? {
                target: "_blank",
                rel: "noopener noreferrer",
              }
            : {})}
          {...rest}
        >
          {children}
        </a>
      );
    }
    return (
      <RouterLink ref={ref} to={to} {...rest}>
        {children}
      </RouterLink>
    );
  }
);

UnstyledLink.displayName = "UnstyledLink";

export default UnstyledLink;
