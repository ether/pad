/* Inspired by JUITTER 1.0.0 BY RODRIGO FANTE */

function createSearchURL(){
 var url = "http://localhost:9000/ep/tag/?format=json";
 /* We could in theory do something here. */
 return url; }

/* This could take an argument, i.e. to
   say whether the loop has run before or not?
   (Trying it.)*/
function runMainLoop(mode){
    var mytimer;
    /* initialization mode: start working on the realtimedata div */
    if(mode===0)
        jQuery("#realtimedata").html("");
    /*fetch new data and do stuff, but first reset the msgNb...*/
    msgNb = 0;
    runAjaxStuff();
    /*once that's done, call self again.*/
    mytimer=setTimeout(runMainLoop(1),seconds*1000);
}

function runAjaxStuff(){
 jQuery.ajax({
   url: createSearchURL(),
   type: 'GET',
   dataType: 'jsonp',
   timeout: 1000,
   error: function(){jQuery("#realtimedata").html("fail#"); },
   success: doSomethingWithJSON(json);});}

 /* This function inserts some HTML to format items
 -- Perhaps we'd do that directly in the ejs file instead? */

function doSomethingWithJSON(json){
    jQuery("<ul>")
        .attr('class', "allmatches")
        .prependTo("#realtimedata");
    /* Mark up each of the matching pads. */
    jQuery.each(json.matchingPads, function(i,item){
            /* Inserting and marking up the items we obtained. */
            if (item.text != "undefined") {
                /*We collect all of the tags into one string --
                  better markup could be used to be consistent with
                  the usual look & feel.*/
                var tagString = "";
                for (j = 0; j < matchingPads[i].TAGS.length; j++){
                    tagString = tagString +
	                "<a href=http://localhost/ep/tag?query=" +
                        matchingPads[i].TAGS[j] +
                        "class=padtag + title=" + matchingPads[i].TAGS[j] +
                        " matches>" + matchingPads[i].TAGS[j] + "</a>";
                }
                jQuery("<li>")
                    .html("<a href=http://localhost:9000/"+item.ID+">"
                          +item.ID+"</a>"+
                          "&nbsp; " + item.lastWriteTime +
                          "<br>" + tagString)
                    .attr('id', "matchingpad"+msgNb)
                    .appendTo("#realtimedata");

                jQuery('#matchingpad'+msgNb).hide();
                jQuery('#matchingpad'+msgNb).show("slow");

                msgNb++; }}});
    /* After dealing with the new stuff, remove any old
       entries */
    /*Do we have to deal with some kind of reset of msgNb?*/
    if(msgNb>=maxNumMessages){
        jQuery(".matchingpad").each(
            function(k,elemLI){
                if(k >= maxNumMessages)
                    jQuery(this).hide("slow");
            });}
}

// Something like this here:
$(document).ready(function () {
  // do stuff to start things on client side
  // (set up the start, timer, all that stuff)
  var seconds = 15;
  var msgNb = 0;
  var maxNumMessages = 10;
  var timer=setTimeout(runMainLoop(0),seconds*1000);
});

realTimeRecentChanges = new init();
