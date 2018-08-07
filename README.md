# dual-ec-drbg

This is an interactive proof-of-concept of the Dual_EC_DRBG backdoor. It uses an implementation of the 2006 Dual_EC_DRBG algorithm without any additional input. Please see [this redacted NIST paper](https://csrc.nist.gov/publications/detail/sp/800-90a/archive/2012-01-23) for algorithmic details. Project Bullrun also has [a good overview of the subject](https://www.projectbullrun.org/dual-ec/documents/dual-ec-20150731.pdf).

This demonstration shows how a [Shumlow-Ferguson attack](http://rump2007.cr.yp.to/15-shumow.pdf) could be executed to recover the internal state of the pseudorandom number generator after the attacker sees as little as 32 bytes. It attempts to adhere as closely as possible to the actual NIST specifications of the algorithm. However, because finding the actual backdoor used in the paper is computationally hard and would require solving the ECDLP, the demonstration uses its own version of point Q, and you can choose the backdoor d, such that `dQ = P`.

In order to run the demonstration, open your favorite terminal and run these commands:

```
git clone https://github.com/dmhacker/dual-ec-drbg
cd dual-ec-drbg
cargo run
```

You can choose what curve, backdoor, and seed to use by passing them as additional arguments. Use
```
cargo run -- --help
```
for additional help. By default, the program uses the P-256 curve and randomly generated seed and backdoor values. All numbers in the program are displayed in hexadecimal. 

The demo was tested on a Surface Book 2 running an 8th-gen i7-8650 GHz mobile processor (8 cores). In the worst case scenario, when the truncated bits were in the 60000's, the program took about 25 seconds to generate the DRBG's state. You can provide the `--release` flag to cargo to build an optimized version of the program that reduces runtime by about 20% at the cost of additional compilation time.
