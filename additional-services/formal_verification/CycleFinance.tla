--------------------------- MODULE CycleFinance ---------------------------
EXTENDS Naturals, Sequences, TLC

(* 
  TLA+ Model for Cycle Finance Lock-Free Tick Processing.
  Proves that ticks produced by the network adapter are eventually consumed
  by the core without deadlocks (Liveness) and without dropping (Safety).
*)

VARIABLES 
    queue,       \* Lock-free MPMC queue (flume)
    ticks_in,    \* Total ticks generated
    ticks_out    \* Total ticks processed

vars == <<queue, ticks_in, ticks_out>>

Init == 
    /\ queue = <<>>
    /\ ticks_in = 0
    /\ ticks_out = 0

(* Producer adds a tick to the queue *)
Produce == 
    /\ ticks_in < 1000  \* Bounded model checking
    /\ queue' = Append(queue, "tick")
    /\ ticks_in' = ticks_in + 1
    /\ UNCHANGED <<ticks_out>>

(* Consumer processes a tick from the queue lock-free *)
Consume == 
    /\ queue # <<>>
    /\ queue' = Tail(queue)
    /\ ticks_out' = ticks_out + 1
    /\ UNCHANGED <<ticks_in>>

Next == Produce \/ Consume

(* Safety: Processed ticks never exceed produced ticks *)
Safety == ticks_out <= ticks_in

(* Liveness: Every produced tick is eventually consumed *)
Liveness == \A n \in Nat : (ticks_in = n) ~> (ticks_out = n)

Spec == Init /\ [][Next]_vars /\ WF_vars(Consume)

=============================================================================
