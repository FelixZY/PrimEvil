package se.fzy.primevil.ui.theme

import androidx.compose.material3.Typography
import androidx.compose.ui.text.ExperimentalTextApi
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontVariation
import androidx.compose.ui.text.font.FontWeight
import se.fzy.primevil.R

@OptIn(ExperimentalTextApi::class)
val bodyFontFamily =
    FontFamily(
        Font(
            R.font.cinzel_variable_font_wght,
            variationSettings = FontVariation.Settings(FontVariation.weight(300)),
        )
    )
const val BodyFontScale = 1.2

val displayFontFamily =
    FontFamily(
        Font(R.font.cinzel_decorative_regular, FontWeight.Normal),
        Font(R.font.cinzel_decorative_bold, FontWeight.Bold),
        Font(R.font.cinzel_decorative_black, FontWeight.Black),
    )
const val DisplayFontScale = 1.2

// Default Material 3 typography values
val baseline = Typography()

val Typography =
    Typography(
        displayLarge =
            baseline.displayLarge.copy(
                fontFamily = displayFontFamily,
                fontSize = baseline.displayLarge.fontSize * DisplayFontScale,
            ),
        displayMedium =
            baseline.displayMedium.copy(
                fontFamily = displayFontFamily,
                fontSize = baseline.displayMedium.fontSize * DisplayFontScale,
            ),
        displaySmall =
            baseline.displaySmall.copy(
                fontFamily = displayFontFamily,
                fontSize = baseline.displaySmall.fontSize * DisplayFontScale,
            ),
        headlineLarge =
            baseline.headlineLarge.copy(
                fontFamily = displayFontFamily,
                fontSize = baseline.headlineLarge.fontSize * DisplayFontScale,
            ),
        headlineMedium =
            baseline.headlineMedium.copy(
                fontFamily = displayFontFamily,
                fontSize = baseline.headlineMedium.fontSize * DisplayFontScale,
            ),
        headlineSmall =
            baseline.headlineSmall.copy(
                fontFamily = displayFontFamily,
                fontSize = baseline.headlineSmall.fontSize * DisplayFontScale,
            ),
        titleLarge =
            baseline.titleLarge.copy(
                fontFamily = displayFontFamily,
                fontSize = baseline.titleLarge.fontSize * DisplayFontScale,
            ),
        titleMedium =
            baseline.titleMedium.copy(
                fontFamily = displayFontFamily,
                fontSize = baseline.titleMedium.fontSize * DisplayFontScale,
            ),
        titleSmall =
            baseline.titleSmall.copy(
                fontFamily = displayFontFamily,
                fontSize = baseline.titleSmall.fontSize * DisplayFontScale,
            ),
        bodyLarge =
            baseline.bodyLarge.copy(
                fontFamily = bodyFontFamily,
                fontSize = baseline.bodyLarge.fontSize * BodyFontScale,
            ),
        bodyMedium =
            baseline.bodyMedium.copy(
                fontFamily = bodyFontFamily,
                fontSize = baseline.bodyMedium.fontSize * BodyFontScale,
            ),
        bodySmall =
            baseline.bodySmall.copy(
                fontFamily = bodyFontFamily,
                fontSize = baseline.bodySmall.fontSize * BodyFontScale,
            ),
        labelLarge =
            baseline.labelLarge.copy(
                fontFamily = bodyFontFamily,
                fontSize = baseline.labelLarge.fontSize * BodyFontScale,
            ),
        labelMedium =
            baseline.labelMedium.copy(
                fontFamily = bodyFontFamily,
                fontSize = baseline.labelMedium.fontSize * BodyFontScale,
            ),
        labelSmall =
            baseline.labelSmall.copy(
                fontFamily = bodyFontFamily,
                fontSize = baseline.labelSmall.fontSize * BodyFontScale,
            ),
    )
