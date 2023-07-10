package co.mycelium

import co.mycelium.endpoints.Auth.AccessToken
import sttp.tapir._

package object endpoints {
  val base = endpoint.securityIn(auth.bearer[String]()).serverSecurityLogic(jwt => Auth.validate(jwt))
}
