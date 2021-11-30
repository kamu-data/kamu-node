import {Component, EventEmitter, Input, OnChanges, OnInit, Output, SimpleChange, SimpleChanges} from "@angular/core";
import {ClusterNode} from "@swimlane/ngx-graph/lib/models/node.model";
import {DagreNodesOnlyLayout} from "@swimlane/ngx-graph";

@Component({
  selector: 'app-linage-graph',
  templateUrl: './linage-graph.component.html',
})
export class LinageGraphComponent implements OnChanges, OnInit {
  @Input() public view: [number, number];
  @Input() public links: any[];
  @Input() public nodes: any[];
  @Input() public clusters: any[];

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
  public miniMapPosition: any;
  public graphClusters: any[];
  public graphNodes: any[];

  public ngOnInit(): void {
    debugger
    this.graphNodes = this.nodes || [];
    this.graphClusters = this.graphClusters || [];
  }
  public ngOnChanges(changes: SimpleChanges): void {
    debugger
    const clusters: SimpleChange = changes.clusters;
    const nodes: SimpleChange = changes.nodes;
    if (clusters) {
      if (typeof clusters.currentValue !== 'undefined' && clusters.currentValue !== clusters.previousValue) {
        if (typeof clusters.currentValue !== 'undefined') {
          debugger
          this.graphClusters = clusters.currentValue.filter((cluster: ClusterNode) => cluster.childNodeIds && cluster.childNodeIds.length !== 0);
        }
      }
    }
    if (nodes) {
      if (typeof nodes.currentValue !== 'undefined' && nodes.currentValue !== nodes.previousValue) {
        if (typeof nodes.currentValue !== 'undefined') {
          debugger
          this.graphNodes = nodes.currentValue;
        }
      }
    }
  }

  public onClickNode(node: any, label: string): void {
    debugger
    console.log(node);
    this.onClickNodeEvent.emit(label);
  }
}
