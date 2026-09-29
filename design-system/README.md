# Gradus design system

Browser-first visual studies for Gradus and the future home of reusable HTML
components. This component is independent of the Flutter client; a browser
prototype may be ported to Flutter later, but is not a Flutter package.

For the shared Flutter widget contracts, page compositions, and map handoff, see
[the activity UI handoff](prototype/HANDOFF.md). The canonical colours,
typography, spacing and component variants are in the [design tokens](prototype/TOKENS.md).

## Activity-detail prototypes

Open `prototype/activity-detail.html` for the running activity or
`prototype/gym-activity-detail.html` for the strength workout. Both are
phone-focused Modern Palaestra studies with a grounded coach’s brief and
recent-self comparisons. They are independent HTML pages with no shared code.

The prototypes use illustrative data. The running page offers pace, heart-rate,
and elevation chart controls; the strength page offers volume, repetitions, and
RPE controls. They do not parse activity files.

## Activity archive prototype

Open `prototype/activity-archive.html` for a phone-focused activity history with
an April summary and links to the running and strength activity-detail studies.
The data is illustrative; no activities are loaded or saved.

## Workout logging prototype

Open `prototype/gym-workout.html` for an in-progress strength session. It lists
six exercises in order, presents the planned sets, load, and repetitions, and
provides labeled inputs for actual results. A collapsible coach’s brief offers
session tips, exercise-completion controls update progress, and a persistent
action finishes the workout once every set is logged. This page is fully
independent from both activity-detail pages.

### Image attribution

`prototype/assets/discobolus-of-myron.jpg` is a public-domain image from the
1911 *Encyclopædia Britannica*: [Discobolus of Myron on Wikimedia
Commons](https://commons.wikimedia.org/wiki/File:EB1911_Greek_Art_-_Discobolus_of_Myron.jpg).
It depicts a restored version of Myron's Discobolus, photographed by F.
Bruckmann.

`prototype/assets/statue.png` was supplied for the activity-detail design
exploration. Its source and redistribution license are not yet documented, so
it must remain prototype-only until provenance is confirmed.
`prototype/assets/statue-display.png` is its resized, grayscale display copy.

`prototype/assets/running.png` was created and supplied by the project owner
for this prototype. `prototype/assets/running-display.png` is the optimized,
grayscale copy used by the English v2 screen. Confirm final redistribution
terms before promoting the artwork beyond the prototype.

`prototype/assets/gym.png` was supplied for the strength activity-detail study;
`prototype/assets/gym-display.png` is its optimized grayscale display copy. Its
source and redistribution license are not yet documented, so it must remain
prototype-only until provenance is confirmed.
