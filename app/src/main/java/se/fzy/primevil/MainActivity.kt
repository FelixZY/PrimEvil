package se.fzy.primevil

import android.icu.text.DecimalFormat
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.SizeTransform
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SuggestionChip
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.core.splashscreen.SplashScreen.Companion.installSplashScreen
import kotlin.coroutines.cancellation.CancellationException
import kotlin.time.Duration.Companion.milliseconds
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlinx.datetime.Clock
import kotlinx.datetime.Instant
import se.fzy.primevil.ui.theme.PrimEvilTheme

class MainActivity : ComponentActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        installSplashScreen()
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()

        setContent {
            PrimEvilTheme {
                Scaffold(modifier = Modifier.fillMaxSize()) { innerPadding ->
                    Content(
                        modifier = Modifier.fillMaxSize().padding(innerPadding),
                        primer = Primer,
                    )
                }
            }
        }
    }
}

@Composable
fun Content(modifier: Modifier = Modifier, primer: Primer = remember { Primer }) {
    Box(modifier = modifier, contentAlignment = Alignment.Center) {
        Column(
            modifier = Modifier.fillMaxWidth(),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center,
        ) {
            var isCrunching by remember { mutableStateOf(false) }
            var prime by remember { mutableLongStateOf(2) }
            var primeCount by remember { mutableIntStateOf(1) }
            val snapshotInterval = 250.milliseconds

            Spacer(modifier = Modifier.weight(1f))
            AnimatedCounter(prime)
            Text(
                buildAnnotatedString {
                    append("Prime no. ")
                    withStyle(SpanStyle(fontWeight = FontWeight.Black)) {
                        append(DecimalFormat.getIntegerInstance().format(primeCount))
                    }
                },
                modifier = Modifier.fillMaxWidth(),
                style = MaterialTheme.typography.bodyMedium,
                textAlign = TextAlign.Center,
            )

            Spacer(modifier = Modifier.weight(1f))

            var lastCrunchCallback by remember { mutableStateOf(Instant.DISTANT_PAST) }
            Controls(
                isCrunching = isCrunching,
                onCrunchStart = { isCrunching = true },
                onCrunchEnd = {
                    isCrunching = false

                    if (it == null) return@Controls

                    prime = it.lastPrime
                    primeCount = it.totalPrimes
                },
                onEachCrunch = {
                    if (Clock.System.now() - lastCrunchCallback > snapshotInterval) {
                        prime = it.lastPrime
                        primeCount = it.totalPrimes
                        lastCrunchCallback = Clock.System.now()
                    }
                },
                primer = primer,
            )
        }
    }
}

@Composable
fun AnimatedCounter(value: Long) {
    AnimatedContent(
        targetState = DecimalFormat.getIntegerInstance().format(value),
        label = "counter",
        transitionSpec = {
            slideInVertically { -it } + fadeIn() togetherWith
                fadeOut() using
                SizeTransform(clip = false)
        },
    ) {
        Text(
            it.toString(),
            modifier = Modifier.fillMaxWidth(),
            style = MaterialTheme.typography.displayLarge,
            textAlign = TextAlign.Center,
        )
    }
}

@Composable
@Preview
fun Controls(
    isCrunching: Boolean = false,
    onCrunchStart: () -> Unit = {},
    onCrunchEnd: (Primer.CrunchResult?) -> Unit = {},
    onEachCrunch: (Primer.CrunchResult) -> Unit = {},
    primer: Primer = remember { Primer },
) {
    Column(horizontalAlignment = Alignment.CenterHorizontally) {
        CrunchPresets(
            modifier = Modifier.padding(horizontal = 16.dp),
            isCrunching = isCrunching,
            onCrunchStart = onCrunchStart,
            onCrunchEnd = onCrunchEnd,
            onEachCrunch = onEachCrunch,
            primer = primer,
        )
        Spacer(modifier = Modifier.height(16.dp))
        CrunchButton(
            modifier = Modifier.padding(horizontal = 16.dp),
            isCrunching = isCrunching,
            onCrunchStart = onCrunchStart,
            onCrunchEnd = onCrunchEnd,
            onEachCrunch = onEachCrunch,
            primer = primer,
        )
    }
}

private val presetButtons =
    arrayOf(
        1,
        5,
        10,
        50,
        100,
        500,
        1_000,
        5_000,
        10_000,
        25_000,
        50_000,
        100_000,
        500_000,
        1_000_000,
        1_500_000,
    )

@Composable
@Preview
fun CrunchPresets(
    modifier: Modifier = Modifier,
    isCrunching: Boolean = false,
    onCrunchStart: () -> Unit = {},
    onCrunchEnd: (Primer.CrunchResult?) -> Unit = {},
    onEachCrunch: (Primer.CrunchResult) -> Unit = {},
    primer: Primer = remember { Primer },
) {
    val scrollState = rememberScrollState()
    val scope = rememberCoroutineScope()
    AnimatedContent(
        !isCrunching,
        transitionSpec = { fadeIn() togetherWith fadeOut() },
        label = "enabled",
    ) { enabled ->
        Row(
            modifier = Modifier.horizontalScroll(scrollState).then(modifier),
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            val vibrator = rememberVibrator()
            presetButtons.forEach { presetValue ->
                SuggestionChip(
                    enabled = enabled,
                    onClick = {
                        vibrator.vibrate(20.milliseconds)
                        onCrunchStart()
                        scope.launch(Dispatchers.IO) {
                            var result = Primer.CrunchResult(2, 0)

                            primer.crunch(
                                minOf(presetValue, 5000),
                                OnChunkListener { firstPrimeIndex, chunk ->
                                    result =
                                        Primer.CrunchResult(
                                            chunk.last(),
                                            firstPrimeIndex + chunk.size,
                                        )
                                    scope.launch(Dispatchers.Main) { onEachCrunch(result) }
                                    return@OnChunkListener isActive &&
                                        firstPrimeIndex + chunk.size < presetValue
                                },
                            )
                            withContext(Dispatchers.Main) { onCrunchEnd(result) }
                        }
                    },
                    label = { Text("+" + DecimalFormat.getIntegerInstance().format(presetValue)) },
                )
            }
        }
    }
}

@Composable
fun CrunchButton(
    modifier: Modifier = Modifier,
    isCrunching: Boolean = false,
    onCrunchStart: () -> Unit = {},
    onCrunchEnd: (Primer.CrunchResult?) -> Unit = {},
    onEachCrunch: (Primer.CrunchResult) -> Unit = {},
    primer: Primer = remember { Primer },
) {
    val vibrator = rememberVibrator()
    val crunchButtonInteractionSource = remember { MutableInteractionSource() }
    val isCrunchButtonPressed by crunchButtonInteractionSource.collectIsPressedAsState()
    var crunchButtonJob by remember { mutableStateOf<Job?>(null) }

    LaunchedEffect(isCrunchButtonPressed) {
        if (!isCrunchButtonPressed) {
            crunchButtonJob?.cancelAndJoin()
            return@LaunchedEffect
        }
        crunchButtonJob = launch {
            onCrunchStart()
            withContext(Dispatchers.IO) {
                var lastResult: Primer.CrunchResult? = null
                var vibrationStartTimestamp = Instant.DISTANT_PAST
                try {
                    primer.crunch(
                        2500,
                        OnChunkListener { firstPrimeIndex, chunk ->
                            if (Clock.System.now() - vibrationStartTimestamp > 50.milliseconds) {
                                vibrator.vibrate(40.milliseconds)
                                vibrationStartTimestamp = Clock.System.now()
                            }

                            lastResult =
                                Primer.CrunchResult(chunk.last(), firstPrimeIndex + chunk.size)
                            launch(Dispatchers.Main) { onEachCrunch(lastResult!!) }
                            return@OnChunkListener isActive
                        },
                    )
                    withContext(Dispatchers.Main) { onCrunchEnd(lastResult) }
                } catch (_: CancellationException) {}
                onCrunchEnd(lastResult)
            }
        }
    }

    val shakeAnimationProperties = rememberShakeAnimation(isCrunchButtonPressed)
    AnimatedContent(
        !isCrunching || isCrunchButtonPressed,
        transitionSpec = { fadeIn() togetherWith fadeOut() },
        label = "enabled",
    ) { enabled ->
        Button(
            modifier =
                Modifier.graphicsLayer(
                        scaleX = shakeAnimationProperties.scale,
                        scaleY = shakeAnimationProperties.scale,
                        rotationZ = shakeAnimationProperties.rotationZ,
                        translationX = shakeAnimationProperties.translationX,
                        translationY = shakeAnimationProperties.translationY,
                    )
                    .then(modifier),
            enabled = enabled,
            onClick = {},
            interactionSource = crunchButtonInteractionSource,
        ) {
            Box(modifier = Modifier.padding(horizontal = 50.dp, vertical = 25.dp)) {
                Text("Crunch!", fontSize = 40.sp)
            }
        }
    }
}

@Composable
fun rememberShakeAnimation(isActive: Boolean): ShakeAnimationProperties {
    val infiniteTransition = rememberInfiniteTransition(label = "transition")
    val scale by
        infiniteTransition.animateFloat(
            initialValue = if (isActive) 1.05f else 1f,
            targetValue = if (isActive) 1.15f else 1f,
            animationSpec =
                infiniteRepeatable(
                    animation = tween(@Suppress("MagicNumber") 67, easing = LinearEasing),
                    repeatMode = RepeatMode.Reverse,
                ),
            label = "rotation",
        )
    val rotationZ by
        infiniteTransition.animateFloat(
            initialValue = if (isActive) -1f else 0f,
            targetValue = if (isActive) 1f else 0f,
            animationSpec =
                infiniteRepeatable(
                    animation = tween(@Suppress("MagicNumber") 37, easing = LinearEasing),
                    repeatMode = RepeatMode.Reverse,
                ),
            label = "rotation",
        )
    val translationX by
        infiniteTransition.animateFloat(
            initialValue = if (isActive) -10.dp.value else 0f,
            targetValue = if (isActive) 10.dp.value else 0f,
            animationSpec =
                infiniteRepeatable(
                    animation = tween(@Suppress("MagicNumber") 71, easing = LinearEasing),
                    repeatMode = RepeatMode.Reverse,
                ),
            label = "translationX",
        )
    val translationY by
        infiniteTransition.animateFloat(
            initialValue = if (isActive) -10.dp.value else 0f,
            targetValue = if (isActive) 10.dp.value else 0f,
            animationSpec =
                infiniteRepeatable(
                    animation = tween(@Suppress("MagicNumber") 59, easing = LinearEasing),
                    repeatMode = RepeatMode.Reverse,
                ),
            label = "translationY",
        )

    return remember(scale, rotationZ, translationX, translationY) {
        ShakeAnimationProperties(scale, rotationZ, translationX, translationY)
    }
}

data class ShakeAnimationProperties(
    val scale: Float,
    val rotationZ: Float,
    val translationX: Float,
    val translationY: Float,
)

@Preview(showBackground = true)
@Composable
fun GreetingPreview() {
    PrimEvilTheme { Controls() }
}
