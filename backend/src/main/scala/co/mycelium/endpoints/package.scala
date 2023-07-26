package co.mycelium

import sttp.tapir._

package object endpoints {
  val base = endpoint
    .securityIn(auth.bearer[String]())
    .serverSecurityLogic(Auth.validate)
}
