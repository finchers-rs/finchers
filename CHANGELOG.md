<a name="0.6.0"></a>
## 0.6.0 (2017-12-11)


#### Features

* **endpoint:**
  *  add a method 'with_type' ([7d3e4cec](https://github.com/finchers-rs/finchers/commit/7d3e4cecc366c600e6fef04583eeb134d00e018f))
  *  make the error type of some endpoints generic ([43229e02](https://github.com/finchers-rs/finchers/commit/43229e02bc4aa2d25e4633b6f3e1e3f1d775e7b1), breaks [#](https://github.com/finchers-rs/finchers/issues/))
  *  rename `Path` and `PathSeq` to `PathParam` and `PathParams`, and so on ([1facb7d9](https://github.com/finchers-rs/finchers/commit/1facb7d9cb1d10e2daafa508f68b7faa50bc947e), breaks [#](https://github.com/finchers-rs/finchers/issues/))
  *  add missing trait bounds ([caf35bad](https://github.com/finchers-rs/finchers/commit/caf35bad6a75ffc2ac9438582823777898017391))
* **path:**  add a new endpoint `PathSegment` to represents the matcher of a path segment ([f6c59a74](https://github.com/finchers-rs/finchers/commit/f6c59a742d86d6b7627fcf371776b35414eac67c), breaks [#](https://github.com/finchers-rs/finchers/issues/))

#### Bug Fixes

* **context:**  change return type of collect_ramaining_segments() ([f9bfbb23](https://github.com/finchers-rs/finchers/commit/f9bfbb236884f4099152d6128bf551d84b43d865))
* **endpoint:**  ensure that the length of remaining path segments be equal to zero, in `MatchMethod` ([1aad4a35](https://github.com/finchers-rs/finchers/commit/1aad4a353641ac426d229f79dbc77cdb939014da))
* **json:**  make the support for JSON parsing/responder deprecated ([a605da89](https://github.com/finchers-rs/finchers/commit/a605da8933659c602a2e793e6520c9e1d9a76776), breaks [#](https://github.com/finchers-rs/finchers/issues/))

#### Breaking Changes

* **endpoint:**
  *  make the error type of some endpoints generic ([43229e02](https://github.com/finchers-rs/finchers/commit/43229e02bc4aa2d25e4633b6f3e1e3f1d775e7b1), breaks [#](https://github.com/finchers-rs/finchers/issues/))
  *  rename `Path` and `PathSeq` to `PathParam` and `PathParams`, and so on ([1facb7d9](https://github.com/finchers-rs/finchers/commit/1facb7d9cb1d10e2daafa508f68b7faa50bc947e), breaks [#](https://github.com/finchers-rs/finchers/issues/))
* **json:**
  *  make the support for JSON parsing/responder deprecated ([a605da89](https://github.com/finchers-rs/finchers/commit/a605da8933659c602a2e793e6520c9e1d9a76776), breaks [#](https://github.com/finchers-rs/finchers/issues/))
  * the JSON parsing/responder has been moved to `finchers_json` ([21d44294](https://github.com/finchers-rs/finchers/commit/21d44294f136794ee03f9401f53db69af15debfb))
* **path:**  add a new endpoint `PathSegment` to represents the matcher of a path segment ([f6c59a74](https://github.com/finchers-rs/finchers/commit/f6c59a742d86d6b7627fcf371776b35414eac67c), breaks [#](https://github.com/finchers-rs/finchers/issues/))



<a name="0.5.1"></a>
### 0.5.1 (2017-12-07)


#### Bug Fixes

* **server:**  change the type of `Server::addr` from `&'static str` to `String` ([6c13abb0](https://github.com/finchers-rs/finchers/commit/6c13abb02bf6602d8ae5228f90b2465dfd2318d9))



<a name="0.5.0"></a>
## 0.5.0 (2017-09-17)


#### Bug Fixes

*   modify `FromParam` ([4b1c940e](https://github.com/finchers-rs/finchers/commit/4b1c940e6ced0268bb90450febfad2c97b92265e))
*   move definition of associated constant to `PathExt` ([72d3c9e4](https://github.com/finchers-rs/finchers/commit/72d3c9e46dd124e35fb3372e0e7633b68968b960), breaks [#](https://github.com/finchers-rs/finchers/issues/))
*   remove the associated type `FromBody::Future` ([4ee58c13](https://github.com/finchers-rs/finchers/commit/4ee58c13886499fc005101dd11e47165f76f0c39), breaks [#](https://github.com/finchers-rs/finchers/issues/))
*   add a trait method `FromBody::check_request()` ([7c50c450](https://github.com/finchers-rs/finchers/commit/7c50c4507113feb041488080e20c59ed2e78dd10), breaks [#](https://github.com/finchers-rs/finchers/issues/))
*   define `FromPath` ([51a155b3](https://github.com/finchers-rs/finchers/commit/51a155b338150349919b5b89399ab0e074d5ca70))
*   remove constants from `endpoint::path`, and replace them with the associated const `PathConst::PATH` ([576a5ab6](https://github.com/finchers-rs/finchers/commit/576a5ab6322f0e85c56b9e7e89a8979988ab72f6))
*   use `NoReturn` instead of `FinchersError` ([dfb4d4bc](https://github.com/finchers-rs/finchers/commit/dfb4d4bc9f959ff1f48ef18ba171fc1412c0fba3))
*   remove unnecessary constraints from `With` and `Skip` ([8b71bf00](https://github.com/finchers-rs/finchers/commit/8b71bf003d838fd561a86c14268096967e1a05d6))
*   change trait bound of `Server::run_http()` ([50223aac](https://github.com/finchers-rs/finchers/commit/50223aaca99ffcd55b2986401a85e660dbd4689e), breaks [#](https://github.com/finchers-rs/finchers/issues/))
*   update example ([da30d6dd](https://github.com/finchers-rs/finchers/commit/da30d6dde855fe593c60440f9110afb047a9a35d))
*   add the associated type constraint to `Endpoint::or` ([7aef7bba](https://github.com/finchers-rs/finchers/commit/7aef7bba568c27dd00b0e3a2f12c263ae310d3f2), breaks [#](https://github.com/finchers-rs/finchers/issues/))

#### Features

*   add `Form` and helpers ([d7458d52](https://github.com/finchers-rs/finchers/commit/d7458d52f09981d8f95cc3a939b3e76b5369a7cf))

#### Breaking Changes

*   move definition of associated constant to `PathExt` ([72d3c9e4](https://github.com/finchers-rs/finchers/commit/72d3c9e46dd124e35fb3372e0e7633b68968b960), breaks [#](https://github.com/finchers-rs/finchers/issues/))
*   remove the associated type `FromBody::Future` ([4ee58c13](https://github.com/finchers-rs/finchers/commit/4ee58c13886499fc005101dd11e47165f76f0c39), breaks [#](https://github.com/finchers-rs/finchers/issues/))
*   add a trait method `FromBody::check_request()` ([7c50c450](https://github.com/finchers-rs/finchers/commit/7c50c4507113feb041488080e20c59ed2e78dd10), breaks [#](https://github.com/finchers-rs/finchers/issues/))
*   change trait bound of `Server::run_http()` ([50223aac](https://github.com/finchers-rs/finchers/commit/50223aaca99ffcd55b2986401a85e660dbd4689e), breaks [#](https://github.com/finchers-rs/finchers/issues/))
*   add the associated type constraint to `Endpoint::or` ([7aef7bba](https://github.com/finchers-rs/finchers/commit/7aef7bba568c27dd00b0e3a2f12c263ae310d3f2), breaks [#](https://github.com/finchers-rs/finchers/issues/))



<a name="0.4.0"></a>
## 0.4.0 (2017-08-26)

#### Bug Fixes
*   move ownership of some members in `Context` to outside ([d233ee28](https://github.com/finchers-rs/finchers/commit/d233ee28ca0fcc7eddb28b32ea5684ecb0818ad7))
*   change the name of module `endpoint::param` to `endpoint::query` ([7d8d1d85](https://github.com/finchers-rs/finchers/commit/7d8d1d856b80ecd021dbb80a741fc646d91a7cc0))
*   change the signature of `Endpoint::apply()` ([3a2ea793](https://github.com/finchers-rs/finchers/commit/3a2ea79345e69258ce86229090d6ebf3192f0746), breaks [#](https://github.com/finchers-rs/finchers/issues/))

#### Features
*   add `FromBody::Error` ([bcd1f6b7](https://github.com/finchers-rs/finchers/commit/bcd1f6b71532c76f08768b59ae1c16912e53a8d3))
*   export the handle of event loop ([fd84c9b8](https://github.com/finchers-rs/finchers/commit/fd84c9b8c1e273e658a4db178cb81667cc3a9fc1))
*   add `Then` ([6f44b59b](https://github.com/finchers-rs/finchers/commit/6f44b59ba8297ceec83157b42a3b763694c688b8))
*   add an endpoint: `Value` ([c40b10c2](https://github.com/finchers-rs/finchers/commit/c40b10c27a502d32e727dbc099fcfc99394687ab))
*   add a combinator: `AndThen` ([b81e5689](https://github.com/finchers-rs/finchers/commit/b81e56896f49e1139004374d98a96e37fdda205b))
*   make the error type of `Endpoint` as an associated type, and add some combinators ([edf02ce6](https://github.com/finchers-rs/finchers/commit/edf02ce605b143ccb9ce4ac8b619e72a8992fc0c))
*   redefine the trait `NewEndpoint` and change the receiver of `Endpoint::apply()` ([502502c8](https://github.com/finchers-rs/finchers/commit/502502c8eca45bffe96887a53fbe9e90d793a815), breaks [#](https://github.com/finchers-rs/finchers/issues/))
*   switch to multimap ([cf533f97](https://github.com/finchers-rs/finchers/commit/cf533f9715fd7c438d12baca952d957bca11169f))
*   add responders and set appropriate response headers ([edaa7ce5](https://github.com/finchers-rs/finchers/commit/edaa7ce56416ed24c68cc0f1003201e62a568f19))

#### Breaking Changes
*   redefine the trait `NewEndpoint` and change the receiver of `Endpoint::apply()` ([502502c8](https://github.com/finchers-rs/finchers/commit/502502c8eca45bffe96887a53fbe9e90d793a815), breaks [#](https://github.com/finchers-rs/finchers/issues/))
*   change the signature of `Endpoint::apply()` ([3a2ea793](https://github.com/finchers-rs/finchers/commit/3a2ea79345e69258ce86229090d6ebf3192f0746), breaks [#](https://github.com/finchers-rs/finchers/issues/))



<a name="0.3.0"></a>
## 0.3.0 (2017-08-22)

#### Features
*   add useful endpoints ([90136f74](https://github.com/finchers-rs/finchers/commit/90136f74281507bf001124f9a7f040566973f591))
*   add `json_body()` ([fd188f03](https://github.com/finchers-rs/finchers/commit/fd188f038bee1484835e8ae06bb52602991ee41e))

#### Bug Fixes

*   change the return future of `Or<E1,E2>` ([48aa0402](https://github.com/finchers-rs/finchers/commit/48aa0402282138e5883214e293bcbecfc8aa0334), breaks [#](https://github.com/finchers-rs/finchers/issues/))
*   fix implementation of `Clone`/`Copy` ([8bbd68cd](https://github.com/finchers-rs/finchers/commit/8bbd68cd52e573951c89fb478f697d7b34fc825c))
*   add missing derivations and remove some endpoints ([b49ff951](https://github.com/finchers-rs/finchers/commit/b49ff95162c8218ab94378fae31dfce91364689b))
*   move the location of endpoints ([b151df23](https://github.com/finchers-rs/finchers/commit/b151df233fb16fdea92f2fb85b12a0ce23711e57), breaks [#](https://github.com/finchers-rs/finchers/issues/))
*   remove 'NewEndpoint' ([2057eee7](https://github.com/finchers-rs/finchers/commit/2057eee74d1dd1f844e88f5dcbb2fdb6b1d99e20), breaks [#](https://github.com/finchers-rs/finchers/issues/))
*   change the receiver of 'Endpoint::apply' ([7f0dfd14](https://github.com/finchers-rs/finchers/commit/7f0dfd147afa12dcf3c181aca057b5c9d7274ec3), breaks [#](https://github.com/finchers-rs/finchers/issues/))

#### Breaking Changes

*   change the return future of `Or<E1, E2>` ([48aa0402](https://github.com/finchers-rs/finchers/commit/48aa0402282138e5883214e293bcbecfc8aa0334), breaks [#](https://github.com/finchers-rs/finchers/issues/))
*   move the location of endpoints ([b151df23](https://github.com/finchers-rs/finchers/commit/b151df233fb16fdea92f2fb85b12a0ce23711e57), breaks [#](https://github.com/finchers-rs/finchers/issues/))
*   remove `NewEndpoint` ([2057eee7](https://github.com/finchers-rs/finchers/commit/2057eee74d1dd1f844e88f5dcbb2fdb6b1d99e20), breaks [#](https://github.com/finchers-rs/finchers/issues/))
*   change the receiver of `Endpoint::apply` ([7f0dfd14](https://github.com/finchers-rs/finchers/commit/7f0dfd147afa12dcf3c181aca057b5c9d7274ec3), breaks [#](https://github.com/finchers-rs/finchers/issues/))



<a name="0.2.0"></a>
## 0.2.0  (2017-08-21)


#### Features

*   fix signature of TestCase ([10abe4cd](https://github.com/finchers-rs/finchers/commit/10abe4cdbc01eff63f3ef8fc11771a57c995a356))
*   add helper methods ([629e9ab9](https://github.com/finchers-rs/finchers/commit/629e9ab926e0a72ac84062b5d28c46bc68cefa82))

#### Breaking Changes

*   change the return type of 'FromBody::from_body()' ([a73078ac](https://github.com/finchers-rs/finchers/commit/a73078acb203e5815fb41c3a5aa145900482b56f), breaks [#](https://github.com/finchers-rs/finchers/issues/))
*   change definition of Context and Endpoint::apply ([0dbe4aeb](https://github.com/finchers-rs/finchers/commit/0dbe4aeb3eb58371257dcb03b930f34aaf6a49f9), breaks [#](https://github.com/finchers-rs/finchers/issues/))
* **test:**  add TestCase and fix signature of run_test() ([fd6a9aae](https://github.com/finchers-rs/finchers/commit/fd6a9aae4589697de99aa795173c138799732650), breaks [#](https://github.com/finchers-rs/finchers/issues/))

#### Bug Fixes

*   change the return type of 'FromBody::from_body()' ([a73078ac](https://github.com/finchers-rs/finchers/commit/a73078acb203e5815fb41c3a5aa145900482b56f), breaks [#](https://github.com/finchers-rs/finchers/issues/))
*   change definition of Context and Endpoint::apply ([0dbe4aeb](https://github.com/finchers-rs/finchers/commit/0dbe4aeb3eb58371257dcb03b930f34aaf6a49f9), breaks [#](https://github.com/finchers-rs/finchers/issues/))
* **test:**  add TestCase and fix signature of run_test() ([fd6a9aae](https://github.com/finchers-rs/finchers/commit/fd6a9aae4589697de99aa795173c138799732650), breaks [#](https://github.com/finchers-rs/finchers/issues/))



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




