# Demo Day Script: Blower Demonstration

*Autonomous rover + active tool clearing = supervised autonomy proof*

## Pre-Demo Checklist (T-60 minutes)

### Systems Check
- [ ] Rover battery: >80% (full charge preferred)
- [ ] Blower battery: Fully charged, spare ready
- [ ] Depot console: Running, connected, teleop tested
- [ ] GPS lock: Verify RTK fix or strong GPS signal
- [ ] CAN bus: All motors responding (test forward/reverse)
- [ ] Camera feed: Video streaming to console
- [ ] Watchdog: Active, no spurious e-stops
- [ ] LED controller: Status lights working

### Blower Function Test
- [ ] Mount blower attachment securely
- [ ] Power on blower, test 0% → 50% → 100% → 0%
- [ ] Verify air output at 50% (should move paper cups easily)
- [ ] Verify air output at 100% (should clear debris dramatically)
- [ ] Check for vibration issues or abnormal sounds
- [ ] Kill switch accessible and tested

### Debris Staging
**Primary debris** (use these first):
- Paper cups (6-8, crushed slightly)
- Crumpled paper (not too heavy)
- Plastic bags (weighted with paper)

**Backup debris** (if primary fails):
- Leaves (if available)
- Small cardboard pieces
- Foam packaging material

**Staging location**:
- Keep debris out of sight until demo moment
- Have 3-4 sets pre-arranged for quick resets
- Place in clean container (looks intentional, not sloppy)

### Backup Plans Ready
- [ ] **If blower fails**: Tool display table nearby with blower, plow blade, salt spreader
- [ ] **If rover fails**: Laptop open with Rerun recording of successful autonomous run
- [ ] **If GPS fails**: Teleop demo ready (emphasize fleet ops angle)
- [ ] Backup battery pack for blower within reach
- [ ] Spare rover if available (unlikely, but note location)

### Environment Setup
- [ ] Demo path: 20-30 feet, clear of obstacles
- [ ] Cone markers: Define patrol route visually
- [ ] Operator station: Console visible but not obstructing view
- [ ] Investor standing area: Clear line of sight, safe distance (10+ feet)
- [ ] Lighting: Good visibility for camera feed
- [ ] Noise level: Can speak over blower sound

---

## The Standard Demo Flow (2 minutes)

### 1. The Hook (5 seconds)
**What you say**: "Watch this. Drop something in its path."

**What you do**:
- Point to rover already moving in patrol mode
- Hand investor a paper cup or crumpled paper
- Point to path ahead of rover

**Body language**: Confident smile, casual tone. This is routine for you.

---

### 2. The Engagement (10 seconds)
**What you say**: "Go ahead, toss it anywhere in front of the rover."

**What happens**:
- Investor drops debris in path
- Rover continues approaching (blower at 50%)
- You say nothing yet—let them watch

**Key moment**: Investor is now invested. They participated. They want to see it work.

---

### 3. The Action (30 seconds)
**What you say** (as rover approaches debris):
- "The rover's running an autonomous patrol route."
- "GPS navigation, obstacle avoidance."
- "Blower's at 50% right now—standard operating mode."

**What you do**:
- Watch investor's face, not the rover
- If they look excited, boost blower to 100% when rover is 3 feet from debris
- Say: "Let's turn it up." (grin)

**What happens**:
- Blower clears debris dramatically
- Rover continues past without stopping
- Investor reacts (smile, laugh, "whoa")

**Critical**: If debris doesn't clear, don't panic. See "Failure Modes" below.

---

### 4. The Explanation (30 seconds)
**What you say** (after debris is cleared):
- "That's supervised autonomy. One operator manages ten rovers from the depot."
- "We're starting with sidewalk snow removal—$14 billion market, growing 8% annually."
- "Municipalities pay $150-300 per hour per traditional crew."
- "Our rovers? $30 per hour operating cost, work 24/7, never call in sick."

**What you show** (point to console):
- "This is the depot interface—fleet ops, mission dispatch, teleop backup."
- "Each rover logs sessions, telemetry, coverage maps. Full audit trail."

---

### 5. The Business Pitch (30 seconds)
**What you say**:
- "We're raising $500k to deploy our first three rovers in Minneapolis this winter."
- "Revenue model: $50k per rover per season, contracted with city DPW."
- "Hardware cost: $15k per unit at volume. ROI in first season."
- "We've got pilot agreements with two municipalities already lined up."

**What you give them**:
- One-pager with financials, market size, team
- Business card
- QR code to pitch deck

---

### 6. The Close (15 seconds)
**What you say**:
- "Want to drive it? I can hand you the controls."
- OR: "Got questions about the tech, the business, or the deployment?"

**What you do**:
- If they say yes to driving: teleop demo
- If they ask questions: answer, then get contact info
- If they hesitate: "Let me grab your email—I'll send you the deck."

---

## Operator Patter (Step-by-Step)

### Before Investor Drops Object
"The rover's already running its route autonomously. GPS navigation, obstacle detection, pre-planned path. Go ahead—drop something in front of it. Paper cup, whatever you've got."

**Goal**: Make them complicit. They're not watching a demo; they're testing the system.

---

### While Rover Approaches
"Blower's running at 50% right now—standard mode for snow clearing. Saves battery, reduces noise for residential areas."

**Pause. Let them watch.**

If they look bored: "Want to see full power?"
If they look excited: "Watch this." (Boost to 100%)

---

### When Blower Clears Debris
"There it goes." (Don't oversell. Let the action speak.)

Then immediately: "That's the proof of concept—autonomous navigation plus active tool integration. Not just a moving platform, but a working utility vehicle."

---

### After Successful Demo
"Every run gets logged—coverage maps, tool usage, session recordings. Full transparency for municipal audits."

**Then pivot**: "We're focused on snow removal first, but the platform scales. Leaf collection, salt spreading, street sweeping. Same chassis, swap the tool."

---

## Blower Power Management

### When to Boost to 100%
- Investor looks visibly excited or engaged
- They lean forward, smile, or make eye contact
- Debris is large or heavy (cup vs. paper)
- You want a dramatic moment for a hot lead

**How to boost**:
- Use console UI or CLI: `muni tool power 100`
- Say: "Let me turn it up for you."
- Boost when rover is 3 feet from debris

---

### When to Stay at 50%
- Investor looks skeptical or technical (they want realistic ops)
- Debris is light (paper, leaves)
- You're emphasizing battery efficiency or noise reduction
- Multiple demos in a row (save battery)

**Messaging**: "This is normal operating mode—50% power, 4-hour runtime, residential-friendly."

---

### When to Return to 50%
- Immediately after debris is cleared and rover passes
- Before next demo starts
- When investor asks about battery life or noise

**How**: `muni tool power 50`

---

### Reading the Crowd
**Hot lead** (boost every time):
- Smiling, asking detailed questions, taking photos/video
- Mentions budget, timeline, or procurement process
- Asks: "Can we pilot this?"

**Warm lead** (boost once, then ask):
- Interested but cautious
- Asks about cost, scalability, competition
- Needs convincing

**Tire-kicker** (stay at 50%, keep it short):
- Passive body language, checking phone
- Asks vague questions ("How does it work?")
- Doesn't engage when you hand them debris

---

## Failure Modes & Recovery

### Blower Doesn't Start
**Immediate action**:
1. Stay calm. Say: "Hold on, let me reset the tool." (Don't say "failure" or "broken")
2. Power cycle blower via console
3. If no response in 10 seconds, proceed to fallback

**Fallback demo**:
- "Actually, let me show you the full tool suite while we reset."
- Walk to tool display table
- Show blower, plow blade, salt spreader
- Emphasize tool-swapping: "Quick-release mounts, 5-minute swap."
- Pivot to software: "The real value is the autonomy stack—tools are just attachments."

**Pitch adjustment**:
- Emphasize platform over specific tool
- Show Rerun recording on laptop: "Here's a successful run from yesterday."
- Business case remains the same

---

### Debris Doesn't Clear
**Immediate action**:
1. Don't say "it didn't work"
2. Say: "Let me boost the power." (Even if already at 100%)
3. If still doesn't clear: "Interesting—that's heavier than usual. Let's try something lighter."

**Fallback demo**:
- Have investor drop lighter debris (paper instead of cup)
- Or manually place debris in optimal position for next pass
- Or say: "That's why we test in real conditions. Operator can always intervene."

**Pitch adjustment**:
- Emphasize supervised autonomy: "One operator, ten rovers. If one has an issue, operator takes over remotely."
- Show teleop: "Watch—I can drive it manually from here."
- Business case: Still 1:10 ratio vs. traditional crews

---

### Rover Stops Unexpectedly
**Immediate action**:
1. Check console for e-stop or watchdog trigger
2. Say: "Safety system kicked in—let me check what it detected."
3. If obstacle detected: "There's the obstacle avoidance. It saw something."
4. If e-stop: "Manual e-stop—must've brushed the button."

**Fallback demo**:
- If recoverable (restart bvrd): "Give me 30 seconds."
- If not: Switch to teleop. "Let me show you manual control."
- Show console UI: "This is what the operator sees—ten rovers, one screen."

**Pitch adjustment**:
- Emphasize safety systems as a feature: "We over-engineered the safety. Better to stop than risk property damage."
- Show decision-making: "Operator can assess remotely and decide: resume auto, take manual control, or dispatch a tech."

---

### Battery Dies
**Immediate action**:
1. This should never happen if you followed pre-demo checklist
2. If it does: "Battery's lower than I thought. Let me swap it." (If spare available)
3. If no spare: Fallback to laptop demo

**Fallback demo**:
- Open Rerun recording on laptop
- Show full autonomous session
- Emphasize data: "Every run is logged. Coverage maps, tool usage, energy consumption."
- Show business case: "4-hour runtime per charge, 10-minute swap. One rover can cover 2 miles of sidewalk per shift."

**Pitch adjustment**:
- Emphasize fleet ops: "In production, we'll have spare batteries at the depot. Rovers auto-return for swaps."

---

### General Failure Protocol
1. Never say "broken," "failed," or "shit"
2. Always say: "Let me adjust that," "Interesting," or "Safety system"
3. Have a confident fallback ready within 10 seconds
4. Pivot to what works: software, business case, team, pilot agreements
5. End with: "This is why we're testing. Better to catch issues now than in production."

---

## Reading the Investor

### Hot Lead Signals
- **Body language**: Leaning in, smiling, nodding, taking photos/video
- **Questions**: "Can we pilot this?" "What's your timeline?" "Who else is investing?"
- **Engagement**: Asks to drive it, asks detailed technical questions, introduces colleagues
- **Follow-up**: "Send me the deck." "Let's set up a call." "What's your data room?"

**What you do**:
- Spend extra time (5-10 minutes)
- Offer teleop demo
- Get multiple contact methods (email, phone, LinkedIn)
- Schedule follow-up call on the spot
- Introduce them to technical co-founder if present

---

### Warm Lead Signals
- **Body language**: Attentive but reserved, arms crossed but watching
- **Questions**: "How much does it cost?" "What about competition?" "Why now?"
- **Engagement**: Interested but needs convincing, asks about risks
- **Follow-up**: "I'll think about it." "Send me more info."

**What you do**:
- Give full demo (2-3 minutes)
- Answer objections directly
- Emphasize traction (pilot agreements, LOIs)
- Get email, send deck same day
- Note specific objections for follow-up

---

### Tire-Kicker Signals
- **Body language**: Passive, checking phone, looking around
- **Questions**: Vague ("How does it work?") or off-topic ("What's the rover made of?")
- **Engagement**: Doesn't want to drop debris, doesn't ask follow-up questions
- **Follow-up**: "Thanks, this is cool." (Walks away)

**What you do**:
- Keep it short (1 minute)
- Give one-pager if they want it
- Don't chase them
- Focus energy on next conversation

---

### When to Extend Conversation
- They're asking detailed questions
- They mention budget, timeline, or decision-making process
- They're taking photos or recording video
- They introduce you to someone else
- They ask: "Can I get a closer look?"

**How**: Offer teleop demo, show console UI, walk through tool swap, discuss deployment logistics.

---

### When to Let Them Go
- They're polite but disengaged
- They're waiting for you to stop talking
- They say: "Thanks, I'll check it out."
- They're not asking follow-up questions

**How**: "Thanks for your time. Here's my card—reach out if you have questions." Then turn to next person.

---

## Every 30 Minutes Checklist

### Battery Check
- [ ] Rover battery: >60% (recharge if below)
- [ ] Blower battery: >50% (swap if below)
- [ ] Console laptop: Plugged in

### Blower Function Check
- [ ] Quick test: 0% → 50% → 0%
- [ ] Listen for abnormal sounds
- [ ] Check mount security

### Debris Restage
- [ ] Replace used debris (cups, paper)
- [ ] Keep staging area tidy
- [ ] Restock if running low

### Quick Break
- [ ] Water, bathroom, stretch
- [ ] Check phone for hot lead messages
- [ ] Review notes from last 30 minutes

---

## After Each Conversation

### Capture Lead Info
**Immediately** (use phone notes or notebook):
- Name
- Email (confirm spelling)
- Phone (if offered)
- Company/affiliation
- Interest level: Hot / Warm / Cold

### Quick Notes
- Key questions they asked
- Objections raised
- Specific interests (tech, business, deployment)
- Follow-up actions needed

### Follow-Up Actions
**Hot leads** (within 24 hours):
- Send deck + one-pager
- Schedule follow-up call
- Connect on LinkedIn
- Add to CRM

**Warm leads** (within 48 hours):
- Send deck
- Short personal email addressing their objections
- Connect on LinkedIn

**Cold leads**:
- Add to mailing list
- No immediate follow-up unless they reach out

---

## Mental Checklist for Every Demo

1. **Hook**: "Drop something in its path."
2. **Action**: Let them watch (don't oversell)
3. **Explain**: Supervised autonomy, 1:10 ratio
4. **Pitch**: $500k raise, $14B market, pilot agreements
5. **Close**: "Want to drive it?" or "Got questions?"
6. **Capture**: Name, email, interest level

---

## Key Talking Points (Memorize These)

- **Supervised autonomy**: One operator, ten rovers
- **Market**: $14B sidewalk snow removal, 8% CAGR
- **Unit economics**: $15k hardware, $50k revenue/season, ROI in 1 year
- **Raise**: $500k to deploy 3 rovers in Minneapolis
- **Traction**: 2 pilot agreements with municipalities
- **Team**: Robotics + municipal ops experience
- **Scalability**: Same platform, swap tools (snow, leaves, salt, sweeping)

---

## Final Notes

**Remember**:
- Confidence matters more than perfection
- Failures are "safety systems" or "test conditions"
- Always have a fallback (tool table, Rerun recording, teleop)
- Read the room—hot leads get extra time, tire-kickers get one-pagers
- Capture every lead, follow up fast on hot ones

**Mindset**:
- You're not selling a product; you're solving a municipal problem
- The demo is proof; the business case closes the deal
- Investors bet on teams that execute under pressure
- Every conversation is practice for the next one

**End goal**:
- 10-20 meaningful conversations
- 5-10 warm/hot leads
- 1-2 follow-up calls scheduled on the spot
- Zero injuries, zero damaged equipment

Good luck. You've got this.
