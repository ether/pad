window.onerror = function(msg, url, linenumber) {
  var trace = msg + ' @' + url + ':' +  linenumber + '\n\n' + printStackTrace().join('\n\n');
  if (typeof(console) != "undefined") {
    console.log(trace);
  } else {
    alert(trace);
  }
}
