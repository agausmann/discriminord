# discriminord

Create images that appear differently in Discord light and dark themes, or any
other application with two different background colors.

## Installation

- You can download and compile it with Cargo: `cargo install discriminord`.

- Prebuilt binaries are also available in the [GitHub releases][releases].

## How to use it

1. Pick two images, one to show to your dark mode friends, and one to show to
your light mode enemies.

2. Run `discriminord <dark-image>.png <light-image>.png <output>.png`.

3. Share with your friends.

4. Profit!

For more advanced usage, see `discriminord --help`.

## How it works

The image uses transparency to change the brightness of pixels when the
background color changes. The brightness and transparency of the image pixels
can be thought of as parameters to a linear equation: `y = ax + b`. The
background color is passed as the input `x`, and the output color (as seen by
the viewer) is the output `y`. The alpha channel `a` describes how much the
color will vary between the two backgrounds, and the brightness `b` is a
baseline / offset.

[releases]: https://github.com/agausmann/discriminord/releases
