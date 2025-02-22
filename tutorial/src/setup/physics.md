# Wallride mechanics
After setting up Lucio's controls to be comfortable for you, 
the next step is to understand how his wallriding mechanics fundamentally work. 
This will help with understanding why certain things like staying at the same spot 
while repeatedly wall jumping, or turning multiple sharp corners close together don't work, 
while other unintuitive things like clearing massive overhangs do.

Starting with the most important mechanic, the wallride itself. 
You can start a wallride by going near any vertical wall and pressing jump. 
This will make you wallride along the wall until you either release your jump, 
or a wall drop-off happens. Wallriding gives you an extra 30% speed boost compared to your ground speed. 
Unfortunately, there are some caveats to what walls are actually wallridable and which are not. 
Like already mentioned, the major criterium is the verticality of the wall. 
However, there are 2 exceptions to this easy rule. The first one concerns the edges of maps, 
especially those walls near the void. A lot of those walls do not have correct mapped collision 
and wallride hitboxes and some have none. This means wallriding even the possible map edges 
can be finicky to near impossible at times. The second exception has to do with roofs, 
especially those found on the eastern style maps. While the bunk of the roofs may not be wallridable 
due to being a curved wall, some parts are. This includes almost all the edges of these roofs 
as well as some spots on the middle of the area for some. There's no rule to these middle spots 
and if you want to use them, you'll have to learn them by heart.

The next important mechanic to remember is the wall drop-off. This mechanic will throw you off 
the wall you're currently wallriding when encountering a corner and end your wallride, 
and is based on 2 parameters:

1. The angle of the corner. The sharper the angle is the more likely you'll get thrown off. 
If you're turning 2 corner that are only minimally apart the angles of both corners 
will get interpolated, leading to a potentially bigger angle than any of the 2 corners alone.
2. The speed at which you perform the turn. When trying to turn corners at high speed the probability 
of getting thrown off the wall rises significantly. This is mostly only achievable 
by attaching to the wall shortly before the corner, 
due to wallriding slowing you down to the base wallride speed.

If those 2 combined are too high overwatch will automatically cancel your wallride. 
This is not preventable, so watch out when turning corners.
Now, after learning how a wallride can be started and at what point it will disconnect, 
the next step is to start getting some speed and discovering the extra restrictions 
that applies to wallriding, and especially connecting to a wall. 
First, a wall drop-off is not only an automatic mechanic, it can also be triggered manually. 
This is exceptionally easy to do since all you need to do is release your jump input. 
Now, after a wall drop-off occurred, you have around 0.23 seconds to do a 2nd jump input 
which will give you an additional 2.5 m/s speed and a mid-air jump. 
I will refer to this mid-air jump as a “wall jump” throughout the document.

Jumping off a wall will place restrictions on when and where you can start your next wallride. 
The of which is the wallride cooldown. This mechanic prevents you from starting 
another wallride anywhere for 0.5 seconds. This hard time restriction together with the speed decay 
caps your maximum movement speed at around 16 m/s peak for standard settings 
and 21 m/s peak for gravspeed settings.

The next limitation is the same wall distance. You cannot wallride the same wall twice 
without having sufficient distance between the drop off and connect spot of 2 wallrides. 
The minimum distance is around 2 m. This has implications for scaling a wall, as we'll later see.

As the name implies, the same wall distance only counts for the wallrides at the same wall. 
A wall in OW is determined as any connected, straight piece of wallridable surface. 
This especially means any curved “wall” is split up into many small segments in OWs understanding. 
In practice this means, any 2 walls can be wallrode without any distance between them 
as long as they're different in OWs interpretation of walls.

Now that we know what rules apply to wallriding I want to bring up 2 more things 
that influence how fast you'll ultimately move. The first of which is the direction you're holding. 
Since your maximum mid-air speed is tied to your maximum ground speed, holding anything except W, 
or stick forwards for controller players, will result in you slowing down significantly, 
since the ground movement speed when holding A, D, or S is reduced compared to holding W.

The last thing affecting your speed is your mouse movement. 
However, your mouse movement does not only affect your speed, but is also the critical last component 
which will allow you to easily change direction while wallriding without using WASD. 
Generally speaking, Lucio will jump into whatever direction you're currently facing. 
However, he will not exactly go towards that direction but will be dragging behind. 
This is because your facing direction is not the only thing acting towards Lucio's air movement. 
You can imagine your facing direction being a force, that's dragging Lucio into that direction. 
But there's also another force, the force of your current movement direction 
which applies to Lucio's new movement direction. The direction Lucio will go into at the end 
is a compromise of both those forces. This applies when jumping off a wall as well as moving in the air. 
When jumping off a wall the fact if you're looking up or down also plays a part in where Lucio will go. 
If you look up Lucio will jump higher for the trade-off of some speed. 
If you look down Lucio will jump flatter, also for the trade-off of some speed. 
Looking straight ahead will therefore give you the most speed, 
which means you should only look up / down if the next jump is not possible otherwise.

However, your mouse movement can only affect your direction so much and if the angle between 
the 2 acting forces is too steep Lucio will drastically slow down to allow turning into that direction. 
This steep angle is also another reason why pressing A or D slows you down so much.

Before we move on to the next segment, where we will finally explain Lucio's different techs 
you can do with those mechanics, I highly advise you to try out and get comfortable 
with the things mentioned until now. It is very important to have an understanding about these mechanics 
to be able to easily comprehend why certain things work and what you can and cannot do when wallriding in-game. 
You can pause the video and come back after having tried and understood these mechanics.

## Video
{% embed youtube id="NpEaa2P7qZI" %}