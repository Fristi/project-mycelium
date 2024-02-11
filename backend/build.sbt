scalaVersion := "2.13.12"
name         := "backend"
organization := "co.mycelium"
version      := "1.0"

val doobieVersion = "1.0.0-RC4"

libraryDependencies ++= Seq(
  "org.tpolecat"                  %% "doobie-core"           % doobieVersion,
  "org.tpolecat"                  %% "doobie-postgres"       % doobieVersion,
  "org.tpolecat"                  %% "doobie-hikari"         % doobieVersion,
  "org.tpolecat"                  %% "doobie-postgres-circe" % doobieVersion,
  "org.typelevel"                 %% "cats-tagless-macros"   % "0.15.0",
  "com.github.alonsodomin.cron4s" %% "cron4s-core"           % "0.7.0",
  "org.http4s"                    %% "http4s-dsl"            % "0.23.24",
  "org.http4s"                    %% "http4s-ember-server"   % "0.23.24",
  "com.softwaremill.sttp.tapir"   %% "tapir-http4s-server"   % "1.9.9",
  "com.softwaremill.sttp.tapir"   %% "tapir-json-circe"      % "1.9.9",
  "org.flywaydb"                   % "flyway-core"           % "10.0.1",
  "io.circe"                      %% "circe-generic-extras"  % "0.14.3",
  "ch.qos.logback"                 % "logback-classic"       % "1.4.14",
  "com.github.jwt-scala"          %% "jwt-core"              % "9.4.6",
  "com.github.jwt-scala"          %% "jwt-circe"             % "9.4.6",
  "com.auth0"                      % "jwks-rsa"              % "0.22.1",
  "com.github.fs2-blobstore"      %% "s3"                    % "0.9.12",
  "is.cir"                        %% "ciris"                 % "3.5.0",
  "com.github.cb372"              %% "cats-retry"            % "3.1.0",
  "io.sentry"                      % "sentry-logback"        % "6.34.0",
  "org.postgresql"                 % "postgresql"            % "42.6.0",
  "com.softwaremill.sttp.tapir"   %% "tapir-openapi-docs"    % "1.9.9",
  "com.softwaremill.sttp.apispec" %% "openapi-circe-yaml"    % "0.7.4"
)

Compile / scalacOptions ++= {
  CrossVersion.partialVersion(scalaVersion.value) match {
    case Some((2, n)) if n >= 13 => "-Ymacro-annotations" :: Nil
    case _                       => Nil
  }
}

assembly / assemblyMergeStrategy := {
  case PathList("META-INF", entry) if entry.startsWith("services") => MergeStrategy.concat
  case PathList("META-INF", _)                                     => MergeStrategy.discard
  case _                                                           => MergeStrategy.first
}
