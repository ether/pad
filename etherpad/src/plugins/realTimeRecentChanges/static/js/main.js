/* Inspired by JUITTER 1.0.0 BY RODRIGO FANTE -- Thanks Mr. Juitter :)*/

function init() {
  this.hooks = [];
}


function runMainLoop(){
  setTimeout('runMainLoop()', 5000);
  $("div.realtimedata").empty();
  $.getJSON("http://localbox.info:8080/ep/tag/?format=json?",
      function(data){
      doSomethingWithJSON(data);
  });
}

function doSomethingWithJSON(json){
  if (json != undefined && json != ""){
    var msgNb = 0;
    var maxNumMessages = 10;
    $("div.realtimedata").append('<ul class="allmatches">')

    /* Mark up each of the matching pads. */
    jQuery.each(json.matchingPads, function(i,item){
            /* Inserting and marking up the items we obtained. */
            if (item.ID != undefined) {

                /* We collect all of the tags into one
                /* string -- better markup could be used
                /* to be consistent with the usual look &
                /* feel. */

                var tagString = "";

                for (j = 0; j < json.matchingPads[i].TAGS.length; j++){

                    tagString = tagString + '&nbsp; ' +
                        '<a href=http://localbox.info:8080/ep/tag/?query=' +
                        json.matchingPads[i].TAGS[j] +
                        'class=padtag + title=' + json.matchingPads[i].TAGS[j] +
                        ' matches>' + json.matchingPads[i].TAGS[j] + '</a>';
                }

                $("ul.allmatches").append(
                   '<li id="matchingpad' + msgNb + '">' +
                   '<a href=http://localbox.info:8080/' + item.ID + '>' +
                   item.ID + '</a>' +
                   '&nbsp; ' + item.lastWriteTime +
                   '<br>' + tagString);

                jQuery('matchingpad'+msgNb).hide();
                jQuery('matchingpad'+msgNb).show("slow");
                msgNb++; }});

    /* After dealing with the new stuff, remove any old
    /* entries (Do we have to deal with some kind of reset
    /* of msgNb?) Though frankly I find this somewhat
    /* confusing... Should get out a notebook and trace
    /* through the flow, I guess. */

    if(msgNb>=maxNumMessages){
        jQuery(".matchingpad").each(
            function(k,elemLI){
                if(k >= maxNumMessages)
                    jQuery(this).hide("slow");
            });}
}}

realTimeRecentChanges = new init();

/* This gives me problems!  */

$(document).ready(function () {
  // do stuff to start things on client side
  // (set up the start, timer, all that stuff)
  jQuery("#realtimedata").html("");
  runMainLoop();
}); 
