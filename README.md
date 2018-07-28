# dual-ec-drbg-backdoor

<a><img src="https://raw.githubusercontent.com/dmhacker/dual-ec-drbg-backdoor/master/example.jpg" align="center"></a>

This is an interactive proof-of-concept of the Dual_EC_DRBG backdoor. It uses an implementation of the 2006 Dual_EC_DRBG algorithm without any additional input. Please see [this redacted NIST paper](https://csrc.nist.gov/publications/detail/sp/800-90a/archive/2012-01-23) for algorithmic details. Alternatively, Project Bullrun wrote [a good overview of the subject](https://www.projectbullrun.org/dual-ec/documents/dual-ec-20150731.pdf).

This demonstration shows how a [Shumlow-Ferguson attack](http://rump2007.cr.yp.to/15-shumow.pdf) could be executed to recover the internal state of the pseudorandom number generator after the attacker sees as little as two outputs. It attempts to adhere as closely as possible to the actual NIST specifications of the algorithm. However, because finding the actual backdoor used in the paper is computationally hard and work require solving the ECDLP, the demonstration uses its only version of point Q, and you can choose the backdoor d, such that `dQ = P`.

_Please note that this demo uses [pancurses](https://github.com/ihalila/pancurses), a cross-compatible curses library supporting terminals in both Unix and Windows. It's recommended you run the demo in a large terminal window, so that some of the elliptic curve numbers don't wrap around the edge of the screen or get cut off._
