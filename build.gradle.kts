// Top-level build file where you can add configuration options common to all sub-projects/modules.
plugins {
    alias(libs.plugins.mozilla.rust.android) apply false
    alias(libs.plugins.android.application) apply false
    alias(libs.plugins.jetbrains.kotlin.android) apply false
    alias(libs.plugins.googleServices) apply false
    alias(libs.plugins.firebase.crashlytics) apply false
    alias(libs.plugins.detekt) apply false
    alias(libs.plugins.conventionalCommits)
    alias(libs.plugins.spotless)
}

spotless {
    ratchetFrom("origin/main")

    format("misc") {
        target("*.md", ".gitignore")

        trimTrailingWhitespace()
        indentWithSpaces()
        endWithNewline()
        encoding("utf-8")
    }

    kotlin {
        ktfmt("0.53").kotlinlangStyle().configure {
            target("**/*.kt")
            it.setRemoveUnusedImports(true)
            it.setManageTrailingCommas(true)
        }
    }
}

task("addGitHooks") {
    group = "verification"
    copy {
        from(".hooks/")
        into(".git/hooks/")
    }
}

tasks.getByPath(":app:preBuild").dependsOn(tasks.getByName("addGitHooks"))
