import {ModuleWithProviders, NgModule} from "@angular/core";
import {CommonModule} from "@angular/common";
import {FormsModule} from "@angular/forms";
import {LinageGraphComponent} from "./linage-graph.component";
import {NgxGraphModule} from "@swimlane/ngx-graph";

@NgModule({
    imports: [
        CommonModule,
        FormsModule,
        NgxGraphModule,
    ],
    exports: [LinageGraphComponent],
    declarations: [LinageGraphComponent]
})
export class LinageGraphModule {
    public static forRoot(): ModuleWithProviders<LinageGraphModule> {
        return {ngModule: LinageGraphModule};
    }
}