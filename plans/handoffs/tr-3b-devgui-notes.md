# TR-3b's dev GUI notes — for the dev GUI PR between TR-3b and items 176/177

The owner's notes from testing TR-3b's cards in the dev GUI (2026-10-08, at
#234's review). **Delete this file in the dev GUI PR that lands them.**

## The hover's two columns squeeze "As printed" to a sliver

Hovering Flickerwisp shows "Now" beside "As printed", and the second column
wraps every few characters. Every permanent's hover lays the two columns out
in `ui.horizontal_top` (`devgui/src/app.rs`, `fn hover`), and a wrapped
label takes all the width it is offered, so the first column's rules text
leaves the second whatever is left. Flickerwisp was the first long-text card
hovered; any long rules text does it. The fix gives each column its share
(two equal columns, or a width cap on the first).

## The Waiting panel does not draw CR 610.3's returns yet

The engine's view has them (`ui::waiting::Waiting::until_returns`: what each
watches and what it would return), and `plans/devgui-map.md` already says
they join the Waiting panel as a kind of row.
