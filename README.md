# RLParser
_This parser is not meant to rebuild into a replay file._

Built on top of [Boxcars](https://github.com/nickbabcock/boxcars), and is meant to act as a preprocessing format layer to turn the unusable network frames into a useful format.

Typically an analyzer will use a higher level language like java, python, or typescript. Which will result in quite a few hang ups and speed restrictions. However, with most of this boilerplate being done in the binary you are going to run anyway, you can quickly get on with your business logic. The JSON output from the layer will also be smaller and condensed as it weeds out repition and unused fields.

# Precursor
1. All ball actors are considered "the ball"
2. A player's car actors, car component actors, and camera setting actors<sup>[1]</sup> are flattened to that player.
3. Teams are treated as independent of players
4. Header is reduced to match specific information<sup>[2]

<sup>[1]</sup>_treated as a single set currently, even though technically could be updated mid match_

<sup>[2]</sup>_Highlights may or may not make their way in_
