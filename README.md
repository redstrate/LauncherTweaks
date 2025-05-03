# LauncherTweaks

Tweaks for the official FFXIV launcher.

## Features

* Configure the launcher URL, enabling you to write custom launcher pages.
* Configure the boot and game patching URLs.
* Force the launcher/boot executable to use the system proxy. (The web browser portions of the launcher already use the system proxy.)
* Force the launcher/boot executable to always communicate over HTTP instead of HTTPS, to make it easier to replace with local services.
* Bypasses the (soft) WebView2 requirement introduced in Patch 7.21.
* Disable the boot version check.
* Add extra arguments when launching the game.

## Usage

Grab a build from [GitHub Actions](https://github.com/redstrate/LauncherTweaks/actions) or build the project with the instructions below. Then, place the `winmm.dll` next to the launcher.

To configure the launcher URL, place a `launchertweaks.toml` next to the launcher executable:

```toml
# Must be able to serve a index.html or else the launcher 404s:
launcher_url = "https://launcher.mysite.localhost/"
```

### macOS/Linux

Wine will prefer it's own `winmm.dll` by default, but you can change it the configuration (`winecfg`). Applications like [Bottles](https://usebottles.com/) have dedicated settings for this too.

## Building

We require Nightly to build, but `rust-toolchain.toml` should prefer it by default.

## macOS/Linux

It's possible to build this project outside of Windows. Ensure that you have and build with the correct target (`x86_64-pc-windows-gnu`):

```shell
rustup target add x86_64-pc-windows-gnu
```

If you encounter an error message like:

```shell
error occurred in cc-rs: failed to find tool "x86_64-w64-mingw32-gcc": No such file or directory (os error 2)
```

Then you need to install the MinGW toolchain, as one of our dependencies has to compile C code.

## Tips & Tricks

Check out the [Launcher page on wiki.xiv.zone](https://wiki.xiv.zone/Launcher) for more details about the launcher's functions.

### Logging Proxy

You can use the `winmm_proxy` config option to sniff the non-web browser traffic from the launcher. You need a HTTP proxy capable of logging of course, I personally use [mitmproxy](https://mitmproxy.org).

## Credits

Initially based off of [NotNite](https://github.com/NotNite)'s [benchtweaks](https://github.com/NotNite/benchtweaks/) and also inspiration for the name!

## License

![GPLv3](https://www.gnu.org/graphics/gplv3-127x51.png)

This project is licensed under the GNU General Public License 3.
