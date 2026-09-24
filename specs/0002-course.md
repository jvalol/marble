# 0002 The course

**Status:** draft
**Date:** 2026-09-21

## Goal

Somewhere to roll: platforms, gaps, and a reason to get to the end.

## Behavior

A course is a list of platforms, a start, a goal, and some gems. A platform is a
box: a position, a width, a height and a depth. Nothing is sloped, because
blitzkit collides against boxes.

**Falling off** is the hazard. Below a course's floor level there is nothing, and
a marble that drops past it is out, per spec 0003.

**Gems** sit above platforms and are collected by rolling into them: the marble's
sphere against a small sphere around the gem. A collected gem disappears and is
counted. They are optional, a reason to take the harder path rather than a gate
on finishing.

Platforms may touch, and laying them end to end is how a run continues across
one. They may not overlap: two boxes sharing space put two coplanar faces in
front of the camera, and the depth buffer picks between them pixel by pixel,
which reads as a band of stripes along the seam.

**The goal** is a platform that ends the course when the marble rests on it.

**Checkpoints** are platforms marked as such. Rolling onto one remembers it, and
a marble that falls comes back there rather than at the start.

The course ships as data in the game rather than a file to load. One course,
built to teach its own moves: a wide start, a gap that needs speed, a narrow
ledge that needs restraint, and a climb of stepped platforms to the goal.

## Acceptance criteria

- A course's platforms become collision boxes. — `course::tests::platforms_are_colliders`
- The start and goal are on platforms, not in the air. — `course::tests::the_start_and_goal_rest_on_platforms`
- Rolling into a gem collects it, once. — `course::tests::a_gem_is_collected_once`
- A gem out of reach is not collected. — `course::tests::a_distant_gem_is_left_alone`
- Resting on the goal finishes the course. — `course::tests::the_goal_finishes_the_course`
- Touching a checkpoint remembers it. — `course::tests::a_checkpoint_is_remembered`
- A later checkpoint replaces an earlier one. — `course::tests::the_latest_checkpoint_wins`
- Every gem sits above a platform, so none is unreachable. — `course::tests::every_gem_is_reachable`
- Every platform is reachable from the one before it. — `course::tests::the_course_is_connected`
- No two platforms occupy the same space, which would put two faces in one plane for the depth buffer to argue over. — `course::tests::platforms_do_not_interpenetrate`

### Verified by hand

- The course can be finished, and the first gap is crossable at full speed but
  not from standing.
- The seam between the start pad and the run reads as one clean edge, with no
  band of stripes along it.

## Out of scope

Loading courses from a file, more than one course, moving platforms, and any
hazard that is not a gap.
