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




