build:
    #!nu
    dx build

dev platform="desktop":
    #!nu
    dx serve --hot-patch --{{platform}}

