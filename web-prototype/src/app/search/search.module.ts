import {ModuleWithProviders, NgModule} from "@angular/core";
import {CommonModule} from "@angular/common";
import {SearchComponent} from "./search.component";
import {SearchAdditionalButtonsModule} from "../components/search-additional-buttons/search-additional-buttons.module";
import {FormsModule} from "@angular/forms";
import {DynamicTableModule} from "../components/dynamic-table/dynamic-table.module";

@NgModule({
    imports: [
        CommonModule,
        SearchAdditionalButtonsModule,
        FormsModule,
        DynamicTableModule
    ],
    exports: [SearchComponent],
    declarations: [SearchComponent]
})
export class SearchModule {
    public static forRoot(): ModuleWithProviders<any> {
        return {ngModule: SearchModule};
    }
}