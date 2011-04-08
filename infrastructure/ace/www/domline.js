// THIS FILE IS ALSO AN APPJET MODULE: etherpad.collab.ace.domline
// %APPJET%: import("etherpad.admin.plugins");

/**
 * Copyright 2009 Google Inc.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS-IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

// requires: top
// requires: plugins
// requires: undefined

var domline = {};
domline.noop = function() {};
domline.identity = function(x) { return x; };

domline.addToLineClass = function(lineClass, cls) {
  // an "empty span" at any point can be used to add classes to
  // the line, using line:className.  otherwise, we ignore
  // the span.
  cls.replace(/\S+/g, function (c) {
    if (c.indexOf("line:") == 0) {
      // add class to line
      lineClass = (lineClass ? lineClass+' ' : '')+c.substring(5);
    }
  });
  return lineClass;
}

// if "document" is falsy we don't create a DOM node, just
// an object with innerHTML and className
domline.createDomLine = function(nonEmpty, doesWrap, optBrowser, optDocument) {
  var result = { node: null,
                 appendSpan: domline.noop,
                 prepareForAdd: domline.noop,
                 notifyAdded: domline.noop,
                 clearSpans: domline.noop,
                 finishUpdate: domline.noop,
                 lineMarker: 0 };

  var browser = (optBrowser || {});
  var document = optDocument;

  if (document) {
    result.node = document.createElement("div");
  }
  else {
    result.node = {innerHTML: '', className: ''};
  }

  var html = [];  //html content include all blockref content
  var blockHTML = [];
  /**@pram{Array} a set of options {tag, {attrs}} */
  var noderef = [], blockref = [];
  var preHtml, postHtml;
  var curHTML = null;
  function processSpaces(s) {
    return domline.processSpaces(s, doesWrap);
  }
  var identity = domline.identity;
  var perTextNodeProcess = (doesWrap ? identity : processSpaces);
  var perHtmlLineProcess = (doesWrap ? processSpaces : identity);
  var lineClass = 'ace-line';

  function attr2String(attrs){
     var str = "";
     for(var i in attrs){
       str += " "+ i + "='";
       if(typeof attrs[i] == "object"){ 
          //may only used for style
          var temp = "";
          for(var j in attrs[i]){
            temp = j + ":" + attrs[i][j] + ";";
          }  
          str += temp;
       }else{
           str += attrs[i];
       } 
       str += "'"
     }
     return str;
  }

  function mergeRef(ref){
        //combine same ref element
        var map = {}, index = -1; 
        var ret = [];
        for(var i = 0, len = ref.length; i < len; i++){
            if(!ref[i].tag) continue;
            ref[i].tag = ref[i].tag.toLowerCase();
            index = map[ref[i].tag]; 
            if(isNaN(index)){
                //create new item 
                index = ret.push(ref[i]) - 1;
                map[ref[i].tag] = index;
            }else{
                //merge attributes
                var localAttrs = {}, attr;
                for(var j in ref[i].attrs){
                  if(ret[index].attrs[j] === undefined || (typeof ret[index].attrs[j] != "object")){
                     ret[index].attrs[j] = ref[i].attrs[j];
                  }else{
                     for(var m in ref[i].attrs[j]){
                        ret[index].attrs[j][m] = ref[i].attrs[j][m];
                     }
                  } 
                }
            }
        }
        return ret;
  }

  function ref2html(ref){
      var ret = {preHtml:"", postHtml:""};
      ref = mergeRef(ref);
      for(var i = 0, len = ref.length; i < len; i++){
            ret.preHtml += "<" + ref[i].tag + attr2String(ref[i].attrs) + ">";
            ret.postHtml = "</" + ref[i].tag + ">" + ret.postHtml;
      }
      return ret;
  }

  function flushBlock(){ //tag order ?
       var br = ref2html(blockref);
       html.push(br.preHtml + blockHTML.join("") + br.postHtml);
       blockref = [];
       blockHTML = [];
  }
  /**
   * add two more parameters
   * @param{string} txt current working text
   * @param{string} cls classname
   * @param{Array} attributes text attributes
   * @param{boolean} is the txt is marker for line or .. 
   *
   * e.g
   * <div class="ace-line"><!--node reference-->
   *   some text here <!-- block reference && text content -->
   * </div>
   * <div class="ace-line"><!--node reference-->
   *    <ol>
   *        <li><!--block reference>
   *            <h1>Title Here</h1><--text content-->
   *        </li>
   *        <li><!--block reference-->
   *            Some text here <--text content-->
   *        </li>
   *    </ol>
   * </div>
   *
   * using classname as the parameters for building dom line is deprecated, 
   * attributes may be a better choice.
   * Blockref and noderef used to solve the problem which is first in h1 and ol
   * h1 works in block reference, so it can only change current block
   * however, ol works in node reference, it will influence all the content 
   * This also means noderef(ol) has a high priority than blockref(h1)
   * blockref may be equal to noderef in most cases, except in the above case.
   * the elements in reference are order-independent
   * */
  result.appendSpan = function(txt, cls, attributes, lineMarker) {  
    if(lineMarker){
        result.lineMarker +=txt.length;
        txt = "";
    }
    var orderedlist = !!(/\bace\-orderedlist\b/.exec(cls));
    if (cls.indexOf('list') >= 0 || orderedlist) { 
      var listType = /(?:^| )list:(\S+)/.exec(cls);
      if (listType) {
        listType = listType[1];
        if(orderedlist){
            listType += " ace-orderedlist";
        }
        if (listType) {
          preHtml = '<ul class="list-'+listType+'"><li>'; 
          postHtml = '</li></ul>';
        }
      } else if(orderedlist){
          preHtml = '<ul class="ace-orderedlist"><li>'; 
          postHtml = '</li></ul>';
      }
    }

    var href = null;
    var simpleTags = null;
    if (cls.indexOf('url') >= 0) {
      cls = cls.replace(/(^| )url:(\S+)/g, function(x0, space, url) {
         href = url;
      	 return space+"url";
      });
    }
    if (cls.indexOf('tag') >= 0) {
      cls = cls.replace(/(^| )tag:(\S+)/g, function(x0, space, tag) {
    	if (! simpleTags) simpleTags = [];
    	simpleTags.push(tag.toLowerCase());
    	return space+tag;
      });
    }

    var extraOpenTags = "";
    var extraCloseTags = "";

    var plugins_;
    if (typeof(plugins)!='undefined') {
      plugins_ = plugins;
    } else {
      plugins_ = parent.parent.plugins;
    }

    plugins_.callHook(
      "aceCreateDomLine", {domline:domline, cls:cls}
    ).map(function (modifier) {
      cls = modifier.cls;
      extraOpenTags = extraOpenTags+modifier.extraOpenTags;
      extraCloseTags = modifier.extraCloseTags+extraCloseTags;
    });

    var attStr = "";
    plugins_.callHook(
        "aceCreateStructDomLine", {domline:domline, cls:cls, attributes: attributes}
    ).map(function(modifier){ 
        if(modifier.cls){
            cls += " " + modifier.cls;
        }
        if(modifier.attStr !== undefined){
            attStr += modifier.attStr; 
        }
        if(modifier.noderef){
           noderef = noderef.concat(modifier.noderef);
        }
        if(modifier.blockref){
           blockref = blockref.concat(modifier.blockref);
        }
        if(modifier.extraOpenTags){
            extraOpenTags = extraOpenTags+modifier.extraOpenTags;
        }
        if(modifier.extraCloseTags){
            extraCloseTags = modifier.extraCloseTags+extraCloseTags;
        }
    });

    if ((! txt) && cls) {
      lineClass = domline.addToLineClass(lineClass, cls);
    }
    else if (txt) {
      if (href) {
        extraOpenTags = extraOpenTags+'<a href="'+
    	  href.replace(/\"/g, '&quot;')+'">';
	    extraCloseTags = '</a>'+extraCloseTags;
      }
      if (simpleTags) {
    	simpleTags.sort();
    	extraOpenTags = extraOpenTags+'<'+simpleTags.join('><')+'>';
	    simpleTags.reverse();
    	extraCloseTags = '</'+simpleTags.join('></')+'>'+extraCloseTags;
      }
      var pTxt = perTextNodeProcess(domline.escapeHTML(txt));
      if(txt.length && /\bace\-placeholder\b/.exec(cls)){
        pTxt = ""; //don't display text for object marker
      }
      blockHTML.push('<span class="',cls||'','"'+ attStr +'>',extraOpenTags,
		        pTxt,
                extraCloseTags,'</span>');
    }
  };
  result.clearSpans = function() {
    html = [];
    lineClass = ''; // non-null to cause update
    result.lineMarker = 0;
  };
  function writeHTML() {
    flushBlock();
    var newHTML = perHtmlLineProcess(html.join(''));
    if (! newHTML) {
      if ((! document) || (! optBrowser)) {
        newHTML += '&nbsp;';
      }
      else if (! browser.msie) {
        newHTML += '<br/>';
      }
    }
    if (nonEmpty) {
      newHTML = (preHtml||'')+newHTML+(postHtml||'');
    }
    var ret = ref2html(noderef);
    newHTML = ret.preHtml + newHTML + ret.postHtml;
    noderef = [];
    html = preHtml = postHtml = null; // free memory
    if (newHTML !== curHTML) {
      curHTML = newHTML;
      result.node.innerHTML = curHTML;
    }
    if (lineClass !== null) result.node.className = lineClass;
  }
  result.prepareForAdd = writeHTML;
  result.finishUpdate = writeHTML;
  result.getInnerHTML = function() { return curHTML || ''; };

  return result;
};

domline.escapeHTML = function(s) {
  var re = /[&<>'"]/g; /']/; // stupid indentation thing
  if (! re.MAP) {
    // persisted across function calls!
    re.MAP = {
      '&': '&amp;',
      '<': '&lt;',
      '>': '&gt;',
      '"': '&#34;',
      "'": '&#39;'
    };
  }
  return s.replace(re, function(c) { return re.MAP[c]; });
};

domline.processSpaces = function(s, doesWrap) {
  if (s.indexOf("<") < 0 && ! doesWrap) {
    // short-cut
    return s.replace(/ /g, '&nbsp;');
  }
  var parts = [];
  s.replace(/<[^>]*>?| |[^ <]+/g, function(m) { parts.push(m); });
  if (doesWrap) {
    var endOfLine = true;
    var beforeSpace = false;
    // last space in a run is normal, others are nbsp,
    // end of line is nbsp
    for(var i=parts.length-1;i>=0;i--) {
      var p = parts[i];
      if (p == " ") {
	if (endOfLine || beforeSpace)
	  parts[i] = '&nbsp;';
	endOfLine = false;
	beforeSpace = true;
      }
      else if (p.charAt(0) != "<") {
	endOfLine = false;
	beforeSpace = false;
      }
    }
    // beginning of line is nbsp
    for(var i=0;i<parts.length;i++) {
      var p = parts[i];
      if (p == " ") {
	parts[i] = '&nbsp;';
	break;
      }
      else if (p.charAt(0) != "<") {
	break;
      }
    }
  }
  else {
    for(var i=0;i<parts.length;i++) {
      var p = parts[i];
      if (p == " ") {
	parts[i] = '&nbsp;';
      }
    }
  }
  return parts.join('');
};
