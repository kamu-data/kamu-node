import {ModuleWithProviders, NgModule} from "@angular/core";
import {CommonModule} from "@angular/common";
import {SearchComponent} from "./search.component";
import {SearchAdditionalButtonsModule} from "../components/search-additional-buttons/search-additional-buttons.module";
import {FormsModule} from "@angular/forms";
import {DynamicTableModule} from "../components/dynamic-table/dynamic-table.module";
import {MatChipsModule} from "@angular/material/chips";
import {RepoListModule} from "../components/repo-list-component/repo-list.module";
import {PaginationModule} from "../components/pagination-component/pagination.module";

@NgModule({
    imports: [
        CommonModule,
        SearchAdditionalButtonsModule,
        FormsModule,
        DynamicTableModule,
        PaginationModule,
        MatChipsModule,
        RepoListModule
    ],
    exports: [SearchComponent],
    declarations: [SearchComponent]
})
export class SearchModule {
    public static forRoot(): ModuleWithProviders<SearchModule> {
        return {ngModule: SearchModule};
    }
}