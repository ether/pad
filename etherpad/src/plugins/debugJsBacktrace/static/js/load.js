window.onerror = function() {
  var trace = printStackTrace().join('\n\n');
  if (typeof(console) != "undefined") {
    console.log(trace);
  } else {
    alert(trace);
  }
}
