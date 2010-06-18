/* Inspired by JUITTER 1.0.0 BY RODRIGO FANTE */

 /* Apparently AJAX will know what it's doing if
    everything is defined in some standard way? (See
    conectaEtherpad, below.)  But since I don't know how
    AJAX works, I'm just guessing.  Also, I replaced $
    with jQuery throughout -- I suspect it doesn't make
    any sense, but since I don't know what "$" means,
    I'm plunging ahead...*/

/* We need to create a TIMER and a variable that shows
   whether it is RUNNING.  This is used in temporizador. */

function createSearchURL(){
 var url = "http://localhost:9000/ep/tag/?format=json";
 /* We could in theory do something here. */
 return url; }

/* DON'T do it like this. */

function start(){
 this.conectaEtherpad(1)
 /* "running" will have to be defined in this context. */
 if (timer != undefined && !running)
  this.temporizador(); }

function update(){
 this.conectaEtherpad(2);
 if (timer!=undefined)
  this.temporizador(); }

function delRegister(){
 /* remove the oldest entry on the list */
 if(msgNb>=numMSG){
    jQuery(".twittLI").each(
     function(o,elemLI){
       if(o>=numMSG)
        jQuery(this).hide("slow");
       });  }}

function temporizador(){
 /* live mode timer */
 running=true;
 aTim = timer.split("-");
 if(aTim[0]=="live" && aTim[1].length>0){
         tempo = aTim[1]*1000;
         setTimeout("update()",tempo); }}

function conectaEtherpad(mode){
 jQuery.ajax({
   url: createSearchURL(),
   type: 'GET',
   dataType: 'jsonp',
   timeout: 1000,
   error: function(){ jQuery("#realtimedata").html("fail#"); },
   success: doSomethingWithJSON(json, mode)}); }

  /* This function inserts some HTML to format items
  -- Perhaps we'd do that directly in the ejs file instead? */

function doSomethingWithJSON(json, mode){
  /* initialization */
  if(mode==1)
    jQuery("#realtimedata").html("");
  /* Mark up each of the matching pads. */
  jQuery.each(json.matchingPads, function(i,item){
     if(mode==1 || (i < numMSG)){
        if(i==0){
          tultID = item.id;
          jQuery("<ul></ul>")
           .attr('id', 'twittList'+ultID)
           .attr('class','twittList')
           .prependTo("#"+contDiv);
        }
     /* Marking up the items we obtained. */
     if (item.text != "undefined") {
      var link =  "http://twitter.com/" + item.from_user + "/status/" + item.id;
      var tweet = jQuery.Juitter.filter(item.text);

      jQuery("<li></li>")
         .html(mHTML)
         .attr('id', 'twittLI'+msgNb)
         .attr('class', 'twittLI')
         .appendTo("#twittList"+ultID);

      jQuery('#twittLI'+msgNb).hide();
      jQuery('#twittLI'+msgNb).show("slow");

      /* remove old entries */
      jQuery.Juitter.delRegister();
      msgNb++; }}});}

// Something like this here

$(window).load(function () {
  // do stuff to start things on client side
  // (set up the start, timer, all that stuff)
});