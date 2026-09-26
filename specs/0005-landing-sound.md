# 0005 Landing sound

**Status:** implemented
**Date:** 2026-09-24

## Goal

A soft thud when the marble drops onto a surface, so a landing is something you
hear as well as see. A drop that reads as weight rather than as a bump.

## Behavior

**A landing is the moment the marble comes back to ground**, meaning `on_ground`
going from false to true between one update and the next. Staying on the ground
is not a landing, so a marble rolling along a platform is silent however fast it
goes.

**Impact speed is the downward speed the landing frame moves with**, taken
before the move rather than after it. `Marble::update` zeroes the vertical
velocity against the platform it hit before it recomputes `on_ground`, so the
speed read after a landing is always zero. `Marble::landing` carries the value
out, and the frame that reads it takes it.

**Below 3 units per second there is no sound.** That is a settle, not a landing:
the marble easing onto a platform, crossing a seam between two of them, or
dropping the short distance onto a checkpoint after a fall. A marble falls at 30
units per second squared, so 3 is about a tenth of a second of falling, which is
roughly the height of the marble itself.

**Volume rises with impact speed**, linearly from 0.25 at the threshold to 1.0
at the terminal fall speed of 40. A landing from any height is audible, and the
longest drop in the course is the loudest thing in it. These numbers are a
starting point to be tuned by ear, not a result.

**The sound is positional.** The thud plays from where the marble hit, and the
listener sits where the camera sits, looking where it looks. A landing off to
one side is heard off to that side, and turning the camera turns it.

That was not possible when this spec was first written. `queue_spatial` pinned
the listener's ears at the world origin with no way to move them, and this
course runs out to 100 units on two axes, so a thud at the far end would have
been panned hard and faded to nothing. It played flat instead, and this spec
said so and called for "a listener the engine can aim, which is a change to
blitzkit and a spec of its own". That is now blitzkit's spec 0019, and this is
the game that asked for it.

**The thud is placed one ear spacing from the listener**, in the direction of
the marble, rather than at the marble itself. The spatial output fades a sound
by the square of its distance from the ears, and the camera sits nine units back
by default and twenty at full zoom, so a thud at the marble's own position would
arrive between a hundredth and a four hundredth of its volume. Silent, which is
what the first attempt at this was.

Distance carries nothing here in any case. The camera follows the marble, so it
is always about the same distance away, and only the direction is worth hearing.
Volume stays where it belongs, with the impact speed.

**The ears move before the sound is queued**, not after. The camera catches up
to the marble first, then the listener is set from it, then the thud is
appended. Queued the other way round, every thud would be heard from where the
camera was on the previous frame, which at speed is somewhere else.

**One thud per landing.** A landing that happens on the same frame the run
restarts still plays: the sound follows the marble, not the clock.

**With no audio device the game is silent and plays on**, which is what
blitzkit's sound system already does. Nothing here panics for want of a speaker.

## Acceptance criteria

- A marble coming back to ground counts as a landing. — `marble::tests::landing_is_a_transition`
- A marble already on the ground does not land again. — `marble::tests::resting_does_not_land_again`
- Impact speed survives the platform taking the marble's fall. — `marble::tests::impact_speed_survives_the_landing`
- A longer drop lands harder. — `marble::tests::a_harder_drop_lands_harder`
- The short drop onto a checkpoint after a fall is silent. — `marble::tests::a_checkpoint_drop_is_silent`
- A landing under the threshold is silent. — `thud::tests::a_soft_settle_makes_no_sound`
- The quietest audible landing is still audible. — `thud::tests::the_quietest_landing_is_still_audible`
- A faster landing is louder than a slower one. — `thud::tests::a_harder_landing_is_louder`
- Volume stops rising at the terminal fall speed. — `thud::tests::volume_stops_at_the_cap`
- The sound is finite, decays, and never clips. — `thud::tests::the_thud_is_short_and_finite`
- A quieter landing produces quieter samples. — `thud::tests::a_quieter_thud_is_quieter`

### Verified by hand

- The sound reads as a thud rather than a click or a knock. — run the game and
  roll off the first platform.
- A long drop is loud without clipping, and a short one is present without being
  startling. — fall from the highest platform, then step off the lowest.
- Rolling along a platform and across the seam between two of them stays silent.
- A landing off to one side is heard off to that side, and turning the camera
  brings it round. This is the only check that the ears actually moved:
  blitzkit's spec 0019 tests where they go, and nothing tests that moving them
  reaches a speaker.

## Out of scope

Rolling sound, wall impacts, gems, checkpoints and the goal, all of which are
silent and stay that way here. One sound for every surface, rather than a
different material per platform. Pitch or timbre varying with speed: volume is
the only thing impact changes.

## Implementation notes

The thud is generated rather than loaded: a sine sweeping from 150 Hz down to
55 Hz over 180 milliseconds, fading as it goes, built from rodio's `chirp` and
`fade_out`. So marble ships no audio file and has no `res/` directory, unlike
the other three games. A recorded thud would likely sound better, and swapping
one in means changing `thud::thud` and nothing else.
