<a name="0.4.0"></a>
## 0.4.0 (2017-08-26)

#### Bug Fixes
*   move ownership of some members in `Context` to outside ([d233ee28](https://github.com/ubnt-intrepid/finchers/commit/d233ee28ca0fcc7eddb28b32ea5684ecb0818ad7))
*   change the name of module `endpoint::param` to `endpoint::query` ([7d8d1d85](https://github.com/ubnt-intrepid/finchers/commit/7d8d1d856b80ecd021dbb80a741fc646d91a7cc0))
*   change the signature of `Endpoint::apply()` ([3a2ea793](https://github.com/ubnt-intrepid/finchers/commit/3a2ea79345e69258ce86229090d6ebf3192f0746), breaks [#](https://github.com/ubnt-intrepid/finchers/issues/))

#### Features
*   add `FromBody::Error` ([bcd1f6b7](https://github.com/ubnt-intrepid/finchers/commit/bcd1f6b71532c76f08768b59ae1c16912e53a8d3))
*   export the handle of event loop ([fd84c9b8](https://github.com/ubnt-intrepid/finchers/commit/fd84c9b8c1e273e658a4db178cb81667cc3a9fc1))
*   add `Then` ([6f44b59b](https://github.com/ubnt-intrepid/finchers/commit/6f44b59ba8297ceec83157b42a3b763694c688b8))
*   add an endpoint: `Value` ([c40b10c2](https://github.com/ubnt-intrepid/finchers/commit/c40b10c27a502d32e727dbc099fcfc99394687ab))
*   add a combinator: `AndThen` ([b81e5689](https://github.com/ubnt-intrepid/finchers/commit/b81e56896f49e1139004374d98a96e37fdda205b))
*   make the error type of `Endpoint` as an associated type, and add some combinators ([edf02ce6](https://github.com/ubnt-intrepid/finchers/commit/edf02ce605b143ccb9ce4ac8b619e72a8992fc0c))
*   redefine the trait `NewEndpoint` and change the receiver of `Endpoint::apply()` ([502502c8](https://github.com/ubnt-intrepid/finchers/commit/502502c8eca45bffe96887a53fbe9e90d793a815), breaks [#](https://github.com/ubnt-intrepid/finchers/issues/))
*   switch to multimap ([cf533f97](https://github.com/ubnt-intrepid/finchers/commit/cf533f9715fd7c438d12baca952d957bca11169f))
*   add responders and set appropriate response headers ([edaa7ce5](https://github.com/ubnt-intrepid/finchers/commit/edaa7ce56416ed24c68cc0f1003201e62a568f19))

#### Breaking Changes
*   redefine the trait `NewEndpoint` and change the receiver of `Endpoint::apply()` ([502502c8](https://github.com/ubnt-intrepid/finchers/commit/502502c8eca45bffe96887a53fbe9e90d793a815), breaks [#](https://github.com/ubnt-intrepid/finchers/issues/))
*   change the signature of `Endpoint::apply()` ([3a2ea793](https://github.com/ubnt-intrepid/finchers/commit/3a2ea79345e69258ce86229090d6ebf3192f0746), breaks [#](https://github.com/ubnt-intrepid/finchers/issues/))



<a name="0.3.0"></a>
## 0.3.0 (2017-08-22)

#### Features
*   add useful endpoints ([90136f74](https://github.com/ubnt-intrepid/finchers/commit/90136f74281507bf001124f9a7f040566973f591))
*   add `json_body()` ([fd188f03](https://github.com/ubnt-intrepid/finchers/commit/fd188f038bee1484835e8ae06bb52602991ee41e))

#### Bug Fixes

*   change the return future of `Or<E1,E2>` ([48aa0402](https://github.com/ubnt-intrepid/finchers/commit/48aa0402282138e5883214e293bcbecfc8aa0334), breaks [#](https://github.com/ubnt-intrepid/finchers/issues/))
*   fix implementation of `Clone`/`Copy` ([8bbd68cd](https://github.com/ubnt-intrepid/finchers/commit/8bbd68cd52e573951c89fb478f697d7b34fc825c))
*   add missing derivations and remove some endpoints ([b49ff951](https://github.com/ubnt-intrepid/finchers/commit/b49ff95162c8218ab94378fae31dfce91364689b))
*   move the location of endpoints ([b151df23](https://github.com/ubnt-intrepid/finchers/commit/b151df233fb16fdea92f2fb85b12a0ce23711e57), breaks [#](https://github.com/ubnt-intrepid/finchers/issues/))
*   remove 'NewEndpoint' ([2057eee7](https://github.com/ubnt-intrepid/finchers/commit/2057eee74d1dd1f844e88f5dcbb2fdb6b1d99e20), breaks [#](https://github.com/ubnt-intrepid/finchers/issues/))
*   change the receiver of 'Endpoint::apply' ([7f0dfd14](https://github.com/ubnt-intrepid/finchers/commit/7f0dfd147afa12dcf3c181aca057b5c9d7274ec3), breaks [#](https://github.com/ubnt-intrepid/finchers/issues/))

#### Breaking Changes

*   change the return future of `Or<E1, E2>` ([48aa0402](https://github.com/ubnt-intrepid/finchers/commit/48aa0402282138e5883214e293bcbecfc8aa0334), breaks [#](https://github.com/ubnt-intrepid/finchers/issues/))
*   move the location of endpoints ([b151df23](https://github.com/ubnt-intrepid/finchers/commit/b151df233fb16fdea92f2fb85b12a0ce23711e57), breaks [#](https://github.com/ubnt-intrepid/finchers/issues/))
*   remove `NewEndpoint` ([2057eee7](https://github.com/ubnt-intrepid/finchers/commit/2057eee74d1dd1f844e88f5dcbb2fdb6b1d99e20), breaks [#](https://github.com/ubnt-intrepid/finchers/issues/))
*   change the receiver of `Endpoint::apply` ([7f0dfd14](https://github.com/ubnt-intrepid/finchers/commit/7f0dfd147afa12dcf3c181aca057b5c9d7274ec3), breaks [#](https://github.com/ubnt-intrepid/finchers/issues/))



<a name="0.2.0"></a>
## 0.2.0  (2017-08-21)


#### Features

*   fix signature of TestCase ([10abe4cd](https://github.com/ubnt-intrepid/finchers/commit/10abe4cdbc01eff63f3ef8fc11771a57c995a356))
*   add helper methods ([629e9ab9](https://github.com/ubnt-intrepid/finchers/commit/629e9ab926e0a72ac84062b5d28c46bc68cefa82))

#### Breaking Changes

*   change the return type of 'FromBody::from_body()' ([a73078ac](https://github.com/ubnt-intrepid/finchers/commit/a73078acb203e5815fb41c3a5aa145900482b56f), breaks [#](https://github.com/ubnt-intrepid/finchers/issues/))
*   change definition of Context and Endpoint::apply ([0dbe4aeb](https://github.com/ubnt-intrepid/finchers/commit/0dbe4aeb3eb58371257dcb03b930f34aaf6a49f9), breaks [#](https://github.com/ubnt-intrepid/finchers/issues/))
* **test:**  add TestCase and fix signature of run_test() ([fd6a9aae](https://github.com/ubnt-intrepid/finchers/commit/fd6a9aae4589697de99aa795173c138799732650), breaks [#](https://github.com/ubnt-intrepid/finchers/issues/))

#### Bug Fixes

*   change the return type of 'FromBody::from_body()' ([a73078ac](https://github.com/ubnt-intrepid/finchers/commit/a73078acb203e5815fb41c3a5aa145900482b56f), breaks [#](https://github.com/ubnt-intrepid/finchers/issues/))
*   change definition of Context and Endpoint::apply ([0dbe4aeb](https://github.com/ubnt-intrepid/finchers/commit/0dbe4aeb3eb58371257dcb03b930f34aaf6a49f9), breaks [#](https://github.com/ubnt-intrepid/finchers/issues/))
* **test:**  add TestCase and fix signature of run_test() ([fd6a9aae](https://github.com/ubnt-intrepid/finchers/commit/fd6a9aae4589697de99aa795173c138799732650), breaks [#](https://github.com/ubnt-intrepid/finchers/issues/))



<a name="0.1.2"></a>
### 0.1.2 (2017-08-20)
* update Cargo.toml
* improve documentation comments (#2)

<a name="0.1.1"></a>
### 0.1.1 (2017-08-19)
* fix README.md and crate's description

<a name="0.1.0"></a>
## 0.1.0 (2017-08-19)
* First release




