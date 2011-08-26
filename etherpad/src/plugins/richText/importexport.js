jimport("java.lang.System.out.println");
var spanStyle = ['color', 'backgroundColor', 'fontSize',
    'fontFamily', 'width', 'height'];

/**
 * conver fontSize to font-size
 */
function getCSSRuleName(name){
  return (name || "").replace(/([A-Z])/g,function(_, s){return '-' + s.toLowerCase()})
}

function isSpanStyle(name){
  for(var i = 0, len = spanStyle.length; i < len; i++){
    if (name == spanStyle[i]){
      return true;
    }
  }
  return false;
}


function exportInlineStyle(args){
  var result = {
    isAceObject    : false,
    extraOpenTags  : "",
    extraCloseTags : ""
  }  
  if (args && args.attributes) {
    var useSpan = false;
    var spanStyleList = {};
    for(var i = 0, len = args.attributes.length; i < len; i++) {
      attribName  = args.attributes[i].name;
      attribValue = args.attributes[i].value;
      if (isSpanStyle(attribName)){
        var cssRuleName = getCSSRuleName(attribName);   
        spanStyleList[cssRuleName] = attribValue;
        useSpan = true;
      } else {
        var attStr = "";
        switch (attribName){
          case "imgSrc":
              result.extraOpenTags += "<img src=\"" + attribValue + "\" />";
              break;
          case "aceObject":
              result.isAceObject = true;
              break;
          default:
              break;
        }
      }
    }
  }
  if (useSpan) {
    var openSpan = "<span style=\"";   
    for (var style in spanStyleList){
      openSpan += style + ":" + spanStyleList[style] + ";";
    }
    openSpan += "\">"
    result.extraOpenTags  += openSpan;
    result.extraCloseTags += "</span>";
  }
  return [result];
}


function exportLineMarkerStyle(args){
   var result = {
    extraOpenTags  : "",
    extraCloseTags : ""
  }  
  if (args && args.attributes) {
    var name = "", value = "";
    for(var i = 0, len = args.attributes.length; i < len; i++){
      name  = args.attributes[i].name;
      value = args.attributes[i].value;
      switch(name){
        case "textAlign":
          result.extraOpenTags += "<div style=\"text-align:"+ value + "\">";
          result.extraCloseTags = "</div>" + result.extraCloseTags;
          break;
        case "preDefinedStyle":
          if (value == "blockquote") {
            result.extraOpenTags += "<blockquote class=\"richquotestyle\" " +i //TODO add style sheet support
                  "style=\"background-color: #E5ECF9;border:1px solid gray;margin: 0.5em;\">";
            result.extraCloseTags = "</blockquote>" + result.extraCloseTags;
          } else {
            result.extraOpenTags += "<" + value + ">";
            result.extraCloseTags = "</" + value + ">" + result.extraCloseTags;
          }
          break;
        default:
          break;
      }
    }
  }
  return [result];
}
