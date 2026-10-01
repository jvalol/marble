# 0006 It rolls

**Status:** implemented
**Date:** 2026-10-01

## Goal

The marble actually rolls. It has mass, it spins, the course bounces it, and it
is painted so you can see all of that. Before this the ball was a point with a
velocity that `move_and_slide` stopped against walls: a sliding dot that looked
like a marble because it was round and smooth-shaded.

This is the game that asked for blitzkit's spec 0030, and it is the first to use
it.

## Behavior

**The marble is a `blitzkit::physics::Body`**, not a position and a velocity the
game keeps itself. The body carries position, velocity, spin, radius, inverse
mass, restitution and friction together, and `Marble::body` is it. Nothing in the
game integrates gravity or stops a fall any more; `physics::step` does, once per
update, with the course as the static world.

**The handling stays the game's.** How hard a held direction pushes, what the top
speed is, how little of it works in the air, and how a coast settles are all
spec 0001's rules and they are unchanged. Physics owns what the course does
back. The split is the point: the feel of driving a marble is a design decision
and belongs here, while a bounce off a platform is a fact about spheres and
belongs in the engine.

**Nothing spins the ball directly.** The spin comes out of friction at the
contact point, which is the one place a sphere can pick up rotation. Drive the
marble east and it turns about the axis pointing north, at roughly its speed over
its radius, because that is what a ball not slipping does. The alternative was to
set the spin from the velocity each frame, which reads the same while the marble
is on the ground and is a lie the moment it is in the air or on ice.

Spin is radians a second about the axis it points along. `Marble::facing` adds it
up as a quaternion and is renormalised every step, because a quaternion built
from a thousand small multiplications drifts off the unit sphere. Nothing but
drawing reads `facing`.

**Mass is 1, restitution 0.18, friction 0.9.** Nothing else has mass yet, so the
mass only decides how hard the ball bounces rather than who wins a shove.
Restitution is low on purpose: a marble that bounces far is a marble you are not
driving, and 0.18 is a settle you can see rather than a return you have to wait
out. Friction is high because the course should grip, and friction is what turns
the ball.

**Gravity is still 30 and the fall is still capped at 40.** The cap is applied
after the step rather than inside it, since the engine has no terminal speed and
should not: a readable drop is this game's idea.

**Spec 0001's promises still hold.** Gravity pulls it down, a landing stops the
fall, a graze keeps most of the speed, a long drop has a limit, a coast settles.
Those tests were written against the old movement and they pass against this one
unchanged, which is the only evidence worth having that the swap did not change
the game.

**On the ground is still a reach below the marble**, not whatever physics thinks
a contact is. `is_on_ground` is unchanged and the landing sound reads it the way
spec 0005 says.

### What it is painted

**The marble carries a checker**, eight squares around, generated rather than
loaded. A pattern is the whole of what makes rolling visible: spin on a plain
coloured sphere looks exactly like no spin at all, and that is what the ball
looked like the first time it had real spin and no skin.

A checker rather than stripes, because stripes read as turning about one axis and
as nothing about the others, and a ball on a course turns about all of them. Two
colours, light and dark, both opaque, half the image each.

The pattern is built in `skin::pattern` and wrapped in `skin::checker`. They are
separate because a `TextureData` keeps its pixels to itself, so a pattern only
reachable through the texture is a pattern no test can read.

## Acceptance criteria

- Rolling turns the ball, through nothing but friction. — `marble::tests::a_rolling_marble_turns`
- It turns about the axis the direction of travel implies. — `marble::tests::it_turns_the_way_it_rolls`
- A ball standing still does not turn. — `marble::tests::a_still_marble_does_not_turn`
- A fall back to a checkpoint takes the spin away with the speed. — `marble::tests::a_reset_forgets_the_spin`
- The skin is the size it claims and fully populated. — `skin::tests::it_is_the_size_it_says`
- Neighbouring squares differ, and diagonal ones match. — `skin::tests::next_door_squares_differ`
- The two colours get half the image each. — `skin::tests::it_is_half_one_and_half_the_other`
- Nothing in it is transparent. — `skin::tests::it_is_opaque`

Spec 0001's tests are this spec's acceptance criteria too, since keeping them
passing is what the change had to do:

- Gravity still pulls it down. — `marble::tests::gravity_pulls_it_down`
- A landing still stops the fall. — `marble::tests::landing_stops_the_fall`
- A graze still keeps most of the speed. — `marble::tests::a_graze_keeps_most_of_the_speed`
- The fall still has a limit. — `marble::tests::falling_has_a_limit`
- A coast still settles. — `marble::tests::friction_settles_the_marble`
- A landing still reports its impact speed. — `marble::tests::impact_speed_survives_the_landing`

### Verified by hand

- The ball visibly rotates the way it is travelling, and the checker makes it
  obvious. Run the game and roll in a straight line.
- Rolling, stopping and reversing turn it the way a marble would, with no
  skating and no sudden snap when the direction changes.
- A drop onto a platform settles rather than bouncing away, and a drop from the
  highest platform does not pogo.
- Driving into a wall still slides along it rather than sticking or passing
  through.

## Out of scope

Boxes, which spec 0030 does not do either. Any second body: nothing in the course
moves, so the marble never meets another sphere and the engine's body-against-
body half is untested by this game. Rolling sound. Spin changing the handling, a
marble that can be put into a spin and released, or anything reading `facing`
other than the draw.

## Implementation notes

`Marble::position` stayed as a reader because the rest of the game calls it
everywhere; velocity did not, because only tests read it and they say
`marble.body.velocity`. The body is public, so the readers earn their place or
go.

The draw is `push_textured` with `.with_rotation(self.marble.facing)`. The skin
is loaded once when the game starts and held as a `TextureId`.
