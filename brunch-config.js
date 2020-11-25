// See http://brunch.io for documentation.
exports.files = {
  javascripts: {
    joinTo: {
      'vendor.js': /^(?!app)/, // Files that are not in `app` dir.
      'app.js': /^app/
    }
  },
  stylesheets: {joinTo: 'app.css'}
};

exports.paths = {
  public: 'static'
};

exports.plugins = {
  babel: {presets: ['latest']},
  sass: {include_paths: ['node_module/bootstrap/scss']}
};

exports.npm = {
  npm: {
    enabled: true,
    globals: {
      $: 'jquery',
      jQuery: 'jquery',
      jQueryUjs: 'jquery-ujs'
    }
  }
};
