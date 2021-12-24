module.exports = {
    transpileDependencies: [
        'vuetify'
    ],
    css: {
        extract: false
    },
    filenameHashing: false,
    chainWebpack: config => {
        config.optimization.delete('splitChunks')
    }
}
