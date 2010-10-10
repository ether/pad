/* Inspired by JUITTER 1.0.0 BY RODRIGO FANTE -- Thanks Mr. Juitter :)*/


function RTRCrunMainLoop(){
  setTimeout('RTRCrunMainLoop()', 5000);
  $.getJSON("http://" + host + "/ep/tag/?format=json",
      function(data){
        RTRCdoSomethingWithJSON(data);
  });
}

function RTRCdoSomethingWithJSON(json){
  if (json != undefined && json != ""){
    var msgNb = 0;
    var maxNumMessages = 10;
    $("div.realtimedata").empty();
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
                        '<a href=http://' + host + '/ep/tag/?query=' +
                        json.matchingPads[i].TAGS[j] +
                        ' class="padtag" + title="' + json.matchingPads[i].TAGS[j] + 
                        ' matches">#' + json.matchingPads[i].TAGS[j] + '</a>';
                }

                $("ul.allmatches").append(
                   '<li id="matchingpad' + msgNb + '">' +
                   '<a href=http://'+ host +'/' + item.ID + '>' +
                   item.ID + '</a>' +
                   '&nbsp; ' + item.lastWriteTime +
                   '<br>' + tagString + '<br><br>');

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


  // do stuff to start things on client side
  // (set up the start, timer, all that stuff)
$("div.realtimedata").empty();
RTRCrunMainLoop();