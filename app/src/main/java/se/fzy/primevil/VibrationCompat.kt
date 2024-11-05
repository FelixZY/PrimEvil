package se.fzy.primevil

import android.content.Context
import android.os.Build
import android.os.VibrationEffect
import android.os.Vibrator
import android.os.VibratorManager
import androidx.annotation.RequiresApi
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalContext
import kotlin.time.Duration

interface VibrationCompat {

    fun vibrate(duration: Duration)

    companion object {
        fun of(context: Context): VibrationCompat =
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) VibrationCompatS(context)
            else VibrationCompatN(context)
    }
}

private class VibrationCompatN(context: Context) : VibrationCompat {
    @Suppress("DEPRECATION")
    private val vibratorService = context.getSystemService(Context.VIBRATOR_SERVICE) as Vibrator

    override fun vibrate(duration: Duration) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            vibratorService.vibrate(
                VibrationEffect.createOneShot(
                    duration.inWholeMilliseconds,
                    VibrationEffect.DEFAULT_AMPLITUDE,
                )
            )
        } else {
            vibratorService.vibrate(duration.inWholeMilliseconds)
        }
    }
}

@RequiresApi(Build.VERSION_CODES.S)
private class VibrationCompatS(context: Context) : VibrationCompat {
    private val vibrationManager =
        context.getSystemService(Context.VIBRATOR_MANAGER_SERVICE) as VibratorManager

    override fun vibrate(duration: Duration) {
        vibrationManager.defaultVibrator.vibrate(
            VibrationEffect.createOneShot(
                duration.inWholeMilliseconds,
                VibrationEffect.DEFAULT_AMPLITUDE,
            )
        )
    }
}

@Composable
fun rememberVibrator(): VibrationCompat {
    val context = LocalContext.current
    return remember(context) { VibrationCompat.of(context) }
}
