import {Component, EventEmitter, Input, Output,} from "@angular/core";
import {Edge} from "@swimlane/ngx-graph/lib/models/edge.model";
import {Node} from "@swimlane/ngx-graph/lib/models/node.model";
import { MiniMapPosition } from "ngx-graph/projects/swimlane/ngx-graph/src/lib/enums/mini-map-position.enum";

@Component({
  selector: 'app-linage-graph',
  templateUrl: './linage-graph.component.html',
})
export class LinageGraphComponent {
  @Input() public view: [number, number];
  @Input() public links: Edge[];
  @Input() public nodes: Node[];

  @Output() public onClickNodeEvent: EventEmitter<string> = new EventEmitter();

  public draggingEnabled = false;
  public panningEnabled = true;
  public zoomEnabled = true;
  public zoomSpeed = 0.1;
  public minZoomLevel = 0.1;
  public maxZoomLevel = 4.0;
  public panOnZoom = true;
  public autoZoom = true;
  public autoCenter = false;
  public showMiniMap = true;
  public miniMapPosition: MiniMapPosition = MiniMapPosition.UpperLeft;

  public onClickNode(label: string): void {
    this.onClickNodeEvent.emit(label);
  }
}