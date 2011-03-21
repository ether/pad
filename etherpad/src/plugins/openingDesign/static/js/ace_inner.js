function snipp(){
  var x = 0;
  return function() {
   x += 1;
   return x;
  };
};

snopp = snipp();

function snapp() {
 alert($); 
 setTimeout( function(){document.getElementById('snappider').onclick = snopp();}, 1000);
}
