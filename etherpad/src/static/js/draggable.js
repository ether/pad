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


function makeDraggable(jqueryNodes, eventHandler) {
  jqueryNodes.each(function() {
    var node = $(this);
    var state = {};
    var inDrag = false;
    function dragStart(evt) {
      if (inDrag) {
        return;
      }
      inDrag = true;
      if (eventHandler('dragstart', evt, state) !== false) {
        $(document).bind('mousemove', dragUpdate);
        $(document).bind('mouseup', dragEnd);
      }
      evt.preventDefault();
      return false;
    }
    function dragUpdate(evt) {
      if (! inDrag) {
        return;
      }
      eventHandler('dragupdate', evt, state);
      evt.preventDefault();
      return false;
    }
    function dragEnd(evt) {
      if (! inDrag) {
        return;
      }
      inDrag = false;
      try {
        eventHandler('dragend', evt, state);
      }
      finally {
        $(document).unbind('mousemove', dragUpdate);
        $(document).unbind('mouseup', dragEnd);
        evt.preventDefault();
      }
      return false;
    }
    node.bind('mousedown', dragStart);
  });
}

function makeResizableVPane(top, sep, bottom, minTop, minBottom) {
  if (minTop === undefined) minTop = 0;
  if (minBottom === undefined) minBottom = 0;
  var totalHeight = $(top).height() + $(bottom).height();
  var maxTop = totalHeight - minBottom;

  makeDraggable($(sep), function(eType, evt, state) {
    if (eType == 'dragstart') {
      state.startY = evt.pageY;
      state.topHeight = $(top).height();
      state.bottomHeight = $(bottom).height();
    }
    else if (eType == 'dragupdate') {
      var change = evt.pageY - state.startY;

      var topHeight = state.topHeight + change;
      if (topHeight < minTop) { topHeight = minTop; }
      if (topHeight > maxTop) { topHeight = maxTop; }
      change = topHeight - state.topHeight;

      var bottomHeight = state.bottomHeight - change;

      $(top).css('bottom', 'auto');
      $(top).height(topHeight);
      $(sep).css('top', topHeight + "px");
      $(bottom).css('top', 'auto');
      $(bottom).height(bottomHeight);
    }
  });
}

function makeResizableHPane(left, sep, right, minLeft, minRight) {
  if (minLeft === undefined) minLeft = 0;
  if (minRight === undefined) minRight = 0;
  var totalWidth = $(left).width() + $(right).width();
  var maxLeft = totalWidth - minRight;

  makeDraggable($(sep), function(eType, evt, state) {
    if (eType == 'dragstart') {
      state.startX = evt.pageX;
      state.leftWidth = $(left).width();
      state.rightWidth = $(right).width();
    }
    else if (eType == 'dragupdate') {
      var change = evt.pageX - state.startX;

      var leftWidth = state.leftWidth + change;
      if (leftWidth < minLeft) { leftWidth = minLeft; }
      if (leftWidth > maxLeft) { leftWidth = maxLeft; }
      change = leftWidth - state.leftWidth;

      var rightWidth = state.rightWidth - change;

      $(left).css('right', 'auto');
      $(left).width(leftWidth);
      $(sep).css('left', leftWidth + "px");
      $(right).css('left', 'auto');
      $(right).width(rightWidth);
    }
  });
}
