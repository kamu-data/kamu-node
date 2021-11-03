import {ModuleWithProviders, NgModule} from "@angular/core";
import {CommonModule} from "@angular/common";
import {DatasetComponent} from "./dataset.component";
import {SearchAdditionalButtonsModule} from "../components/search-additional-buttons/search-additional-buttons.module";
import {NgbModule, NgbNavModule} from "@ng-bootstrap/ng-bootstrap";
import {FormsModule} from "@angular/forms";
import {DynamicTableModule} from "../components/dynamic-table/dynamic-table.module";
import {SearchSidenavModule} from "../components/search-sidenav/search-sidenav.module";
import {MatButtonToggleModule} from "@angular/material/button-toggle";
import {NgxGraphModule} from "@swimlane/ngx-graph";
import {LinageGraphModule} from "../components/linage-graph/linage-graph.module";

@NgModule({
    imports: [
        CommonModule,
        FormsModule,
        NgbModule,
        FormsModule,
        NgbNavModule,
        MatButtonToggleModule,
        DynamicTableModule,
        SearchSidenavModule,
        SearchAdditionalButtonsModule,
        NgxGraphModule,
        LinageGraphModule
    ],
  exports: [DatasetComponent],
    declarations: [DatasetComponent]
})
export class DatasetModule {
    public static forRoot(): ModuleWithProviders<DatasetModule> {
    return { ngModule: DatasetModule };
  }
}