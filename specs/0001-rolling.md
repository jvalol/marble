# 0001 Rolling

**Status:** draft
**Date:** 2026-09-21

## Goal

A marble that feels like a marble: it builds up speed, it carries on when you
let go, and it falls.

## Behavior

The marble is a sphere with a radius of 0.4 and a velocity. Holding a direction
accelerates it at 24 units per second squared, up to 9 units per second. Letting
go does not stop it: friction takes 6 units per second off its speed, so it
coasts and settles.

**Directions are relative to the camera.** Forward rolls the marble away from
the camera whatever angle the camera sits at, which is what makes the controls
read the same after the camera swings around.

**Gravity** pulls at 30 units per second squared, and the fall speed is capped
at 40 so a long drop stays readable. Landing on a platform stops the fall and
the marble is on the ground again.

**On the ground** means a platform is within a small distance below the marble.
That is what friction applies to, and what lets the marble be steered: in the
air a player has a quarter of the usual control, enough to save a jump but not
enough to fly.

Movement goes through blitkit's `move_and_slide`, so the marble slides along
walls rather than sticking, and never passes through a platform however fast it
is going.

**Hitting a wall costs the speed into it, not all of it.** A marble grazing a
wall keeps most of its pace; one driven straight in stops.

## Acceptance criteria

- Holding a direction builds speed up to the limit. — `marble::tests::rolling_builds_speed`
- Letting go coasts and settles rather than stopping dead. — `marble::tests::friction_settles_the_marble`
- Directions follow the camera. — `marble::tests::rolling_follows_the_camera`
- With nothing underneath, the marble falls. — `marble::tests::gravity_pulls_it_down`
- Falling speed is capped. — `marble::tests::falling_has_a_limit`
- Landing on a platform stops the fall. — `marble::tests::landing_stops_the_fall`
- On the ground it steers fully, in the air barely. — `marble::tests::the_air_gives_less_control`
- A wall takes the speed into it, not all of it. — `marble::tests::a_graze_keeps_most_of_the_speed`

### Verified by hand

- The marble feels heavy rather than twitchy, and coasting to a stop reads as
  rolling rather than braking.

## Out of scope

Bouncing, spin, the marble visibly rotating as it rolls, ramps and slopes, and
any surface that is not flat. blitkit's collision is boxes, per its spec 0014.
