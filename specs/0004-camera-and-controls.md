# 0004 Camera and controls

**Status:** draft
**Date:** 2026-09-21

## Goal

You can see where you are going, and the controls mean the same thing whichever
way you are facing.

## Behavior

**The camera follows the marble** from behind and above, at a distance of 9 and
a height of 5, looking slightly ahead of it. It lags: it moves toward where it
should be rather than snapping, which keeps a fast roll from jerking the view.

**Turning** is the mouse: dragging with the left button, or moving the mouse at
all once the cursor is locked. The wheel changes the distance, between 5 and 20.

**Locking the cursor** is the space bar. Locked, the pointer disappears and
turning never stops at the screen edge, per blitzkit's spec 0013.

**Rolling** is WASD or the arrow keys, relative to the camera, per spec 0001.

**Escape** quits.

**The camera never passes through a platform.** It is pulled in toward the
marble when a platform is between them, so the marble does not disappear behind
a wall. blitzkit's ray cast against the course's boxes is what finds that.

## Acceptance criteria

- The camera sits behind and above the marble. — `camera::tests::the_camera_trails_the_marble`
- Turning moves it around the marble. — `camera::tests::turning_orbits_the_marble`
- It lags rather than snapping. — `camera::tests::the_camera_lags`
- The wheel changes the distance within its limits. — `camera::tests::the_wheel_changes_the_distance`
- A platform between the camera and the marble pulls the camera in. — `camera::tests::a_wall_pulls_the_camera_in`
- With a clear view it stays at its distance. — `camera::tests::a_clear_view_keeps_the_distance`

### Verified by hand

- Rolling into a corner never leaves the marble hidden behind a wall.
- Turning the camera while rolling does not make the marble change direction
  under the same key.

## Out of scope

A first person view, inverting the mouse, sensitivity settings, and any
cinematic camera at the start or the finish.
