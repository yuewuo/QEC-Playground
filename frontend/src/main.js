import Vue from 'vue'
import App from './App.vue'
import './plugins/element.js'
import ElementUI from 'element-ui'
import 'element-ui/lib/theme-chalk/index.css'
import MathjaxConfig from './plugins/mathjax-config'

Vue.config.productionTip = false
Vue.use(ElementUI)
Vue.prototype.MathjaxConfig = MathjaxConfig

new Vue({
  render: h => h(App),
}).$mount('#app')
