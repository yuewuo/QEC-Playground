let isMathjaxConfig = false

const initMathjaxConfig = () => {
    if (!window.MathJax) {
        return
    }
    window.MathJax.Hub.Config({
        showProcessingMessages: false,
        messageStyle: "none",
        jax: ["input/TeX", "output/HTML-CSS"],
        tex2jax: {
            inlineMath: [["$", "$"], ["\\(", "\\)"]],
            displayMath: [["$$", "$$"], ["\\[", "\\]"]],
            skipTags: ["script", "noscript", "style", "textarea", "pre", "code", "a"]
        },
        "HTML-CSS": {
            availableFonts: ["STIX", "TeX"],
            showMathMenu: false
        }
    })
    isMathjaxConfig = true
}

const MathQueue = function (elementId) {
    if (!window.MathJax) {
        return
    }
    if (!isMathjaxConfig) initMathjaxConfig()  // initialize first
    window.MathJax.Hub.Queue(["Typeset", window.MathJax.Hub, document.getElementById(elementId)])
}

export default {
    isMathjaxConfig,
    initMathjaxConfig,
    MathQueue,
}
